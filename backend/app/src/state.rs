use std::path::PathBuf;
use std::sync::Mutex;

use ocorganize_core::{GroupResult, Strategy};

pub struct ScanCache {
    pub directory: PathBuf,
    pub strategy: Strategy,
    pub grouped: GroupResult,
}

pub struct AppState {
    pub operation_history: Mutex<Vec<PathBuf>>,
    pub scan_cache: Mutex<Option<ScanCache>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            operation_history: Mutex::new(Vec::new()),
            scan_cache: Mutex::new(None),
        }
    }

    pub fn store_scan_cache(
        &self,
        directory: PathBuf,
        strategy: Strategy,
        grouped: GroupResult,
    ) -> Result<(), String> {
        let mut cache = self
            .scan_cache
            .lock()
            .map_err(|_| "Failed to lock app state".to_string())?;
        *cache = Some(ScanCache {
            directory,
            strategy,
            grouped,
        });
        Ok(())
    }

    pub fn take_matching_scan_cache(
        &self,
        directory: &PathBuf,
        strategy: Strategy,
    ) -> Result<Option<GroupResult>, String> {
        let mut cache = self
            .scan_cache
            .lock()
            .map_err(|_| "Failed to lock app state".to_string())?;
        let Some(entry) = cache.as_ref() else {
            return Ok(None);
        };
        if entry.directory == *directory && entry.strategy == strategy {
            let entry = cache.take().expect("scan cache entry must exist");
            return Ok(Some(entry.grouped));
        }
        Ok(None)
    }

    pub fn clear_scan_cache(&self) -> Result<(), String> {
        let mut cache = self
            .scan_cache
            .lock()
            .map_err(|_| "Failed to lock app state".to_string())?;
        *cache = None;
        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
