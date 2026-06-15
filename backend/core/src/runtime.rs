use std::sync::OnceLock;

use sysinfo::System;

const ONE_GIB: u64 = 1024 * 1024 * 1024;
const LOW_CORE_THRESHOLD: usize = 4;
const LOW_MEMORY_BYTES: u64 = 4 * ONE_GIB;
const HIGH_CORE_THRESHOLD: usize = 8;
const HIGH_MEMORY_BYTES: u64 = 8 * ONE_GIB;
const MAX_WALK_THREADS: usize = 16;
const MAX_MOVE_CONCURRENCY: usize = 32;
const MIN_PROGRESS_INTERVAL: usize = 20;
const TARGET_PROGRESS_EVENTS: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceTier {
    Low,
    Standard,
    High,
}

#[derive(Debug, Clone)]
pub struct RuntimeProfile {
    pub cpu_threads: usize,
    pub available_memory_bytes: u64,
    pub tier: PerformanceTier,
    pub walk_threads: usize,
    pub metadata_threads: usize,
    pub move_concurrency: usize,
    pub progress_interval: usize,
    pub undo_flush_batch: usize,
}

impl RuntimeProfile {
    pub fn global() -> &'static Self {
        static PROFILE: OnceLock<RuntimeProfile> = OnceLock::new();
        PROFILE.get_or_init(detect_runtime_profile)
    }

    pub fn from_specs(cpu_threads: usize, available_memory_bytes: u64) -> Self {
        let tier = classify_tier(cpu_threads, available_memory_bytes);
        build_profile(cpu_threads, available_memory_bytes, tier)
    }

    pub fn progress_interval_for_total(&self, total_files: usize) -> usize {
        if total_files == 0 {
            return MIN_PROGRESS_INTERVAL;
        }
        (total_files / TARGET_PROGRESS_EVENTS)
            .max(MIN_PROGRESS_INTERVAL)
            .min(self.progress_interval.max(MIN_PROGRESS_INTERVAL))
    }
}

fn detect_runtime_profile() -> RuntimeProfile {
    if let Ok(raw) = std::env::var("OCORGANIZE_PERF_TIER") {
        let tier = match raw.to_lowercase().as_str() {
            "low" => PerformanceTier::Low,
            "high" => PerformanceTier::High,
            _ => PerformanceTier::Standard,
        };
        let cpu_threads = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(4);
        let available_memory_bytes = read_available_memory_bytes();
        return build_profile(cpu_threads, available_memory_bytes, tier);
    }

    let cpu_threads = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(4);
    let available_memory_bytes = read_available_memory_bytes();
    let tier = classify_tier(cpu_threads, available_memory_bytes);
    build_profile(cpu_threads, available_memory_bytes, tier)
}

fn read_available_memory_bytes() -> u64 {
    let mut system = System::new();
    system.refresh_memory();
    memory_bytes_for_tiering(
        system.available_memory(),
        system.free_memory(),
        system.total_memory(),
    )
}

fn memory_bytes_for_tiering(available: u64, free: u64, total: u64) -> u64 {
    if available >= LOW_MEMORY_BYTES {
        return available;
    }
    if free >= LOW_MEMORY_BYTES {
        return free;
    }
    if total > 0 && available == 0 {
        return total / 2;
    }
    available.max(free)
}

fn classify_tier(cpu_threads: usize, available_memory_bytes: u64) -> PerformanceTier {
    if cpu_threads < LOW_CORE_THRESHOLD || available_memory_bytes < LOW_MEMORY_BYTES {
        PerformanceTier::Low
    } else if cpu_threads >= HIGH_CORE_THRESHOLD && available_memory_bytes >= HIGH_MEMORY_BYTES {
        PerformanceTier::High
    } else {
        PerformanceTier::Standard
    }
}

fn build_profile(
    cpu_threads: usize,
    available_memory_bytes: u64,
    tier: PerformanceTier,
) -> RuntimeProfile {
    let (walk_threads, metadata_threads, move_concurrency, progress_interval, undo_flush_batch) =
        match tier {
            PerformanceTier::Low => (2, cpu_threads.min(4), 1, 5000, 500),
            PerformanceTier::Standard => (
                cpu_threads.min(8),
                cpu_threads,
                cpu_threads.min(8),
                1000,
                2000,
            ),
            PerformanceTier::High => (
                cpu_threads.min(MAX_WALK_THREADS),
                cpu_threads,
                (cpu_threads / 2).clamp(2, MAX_MOVE_CONCURRENCY),
                500,
                5000,
            ),
        };

    RuntimeProfile {
        cpu_threads,
        available_memory_bytes,
        tier,
        walk_threads,
        metadata_threads,
        move_concurrency,
        progress_interval,
        undo_flush_batch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_bytes_for_tiering_prefers_reported_available() {
        let bytes = memory_bytes_for_tiering(6 * ONE_GIB, ONE_GIB, 16 * ONE_GIB);
        assert_eq!(bytes, 6 * ONE_GIB);
    }

    #[test]
    fn memory_bytes_for_tiering_falls_back_to_free_memory() {
        let bytes = memory_bytes_for_tiering(ONE_GIB, 6 * ONE_GIB, 16 * ONE_GIB);
        assert_eq!(bytes, 6 * ONE_GIB);
    }

    #[test]
    fn memory_bytes_for_tiering_estimates_from_total_when_available_unreported() {
        let bytes = memory_bytes_for_tiering(0, ONE_GIB, 24 * ONE_GIB);
        assert_eq!(bytes, 12 * ONE_GIB);
    }

    #[test]
    fn classify_tier_high_when_macos_reports_zero_available() {
        let tier = classify_tier(14, memory_bytes_for_tiering(0, ONE_GIB, 24 * ONE_GIB));
        assert_eq!(tier, PerformanceTier::High);
    }
}
