use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Local};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

use crate::file_type::categorize_path;
use crate::runtime::RuntimeProfile;
use crate::walk::{collect_loose_root_files_with_profile, WalkResult};

const ONE_MB: f64 = 1024.0 * 1024.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Type,
    Size,
    Date,
}

impl Strategy {
    pub fn try_from_str(raw: &str) -> Result<Self, String> {
        match raw {
            "type" | "extension" => Ok(Self::Type),
            "size" => Ok(Self::Size),
            "date" => Ok(Self::Date),
            other => Err(format!("Unknown strategy: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Size => "size",
            Self::Date => "date",
        }
    }
}

pub struct GroupResult {
    pub grouped: HashMap<String, Vec<PathBuf>>,
    pub symlink_skip_count: usize,
    pub organized_folder_skip_count: usize,
    pub hidden_skip_count: usize,
    pub files: Arc<[PathBuf]>,
}

impl Clone for GroupResult {
    fn clone(&self) -> Self {
        Self {
            grouped: self.grouped.clone(),
            symlink_skip_count: self.symlink_skip_count,
            organized_folder_skip_count: self.organized_folder_skip_count,
            hidden_skip_count: self.hidden_skip_count,
            files: Arc::clone(&self.files),
        }
    }
}

impl GroupResult {
    pub fn total_files(&self) -> usize {
        self.grouped.values().map(|files| files.len()).sum()
    }
}

pub fn walk_grouped_files<F>(grouped: &HashMap<String, Vec<PathBuf>>, mut on_file: F)
where
    F: FnMut(&str, &Path),
{
    let mut categories: Vec<_> = grouped.keys().map(String::as_str).collect();
    categories.sort_unstable();
    for category in categories {
        let mut files = grouped[category].clone();
        files.sort();
        for path in &files {
            on_file(category, path);
        }
    }
}

pub fn group_files(directory: &Path, strategy: Strategy) -> GroupResult {
    group_files_with_profile(directory, strategy, RuntimeProfile::global())
}

pub fn group_files_with_profile(
    directory: &Path,
    strategy: Strategy,
    profile: &RuntimeProfile,
) -> GroupResult {
    let WalkResult {
        files,
        symlink_skip_count,
        organized_folder_skip_count,
        hidden_skip_count,
    } = collect_loose_root_files_with_profile(directory, profile);

    let files: Arc<[PathBuf]> = Arc::from(files.into_boxed_slice());
    let pool = ThreadPoolBuilder::new()
        .num_threads(profile.metadata_threads)
        .build()
        .expect("metadata thread pool");

    let grouped = pool.install(|| match strategy {
        Strategy::Type => group_by_file_type(&files),
        Strategy::Size => group_by_size(&files),
        Strategy::Date => group_by_date(&files),
    });

    GroupResult {
        grouped,
        symlink_skip_count,
        organized_folder_skip_count,
        hidden_skip_count,
        files,
    }
}

fn merge_group_maps(
    left: HashMap<String, Vec<PathBuf>>,
    right: HashMap<String, Vec<PathBuf>>,
) -> HashMap<String, Vec<PathBuf>> {
    if left.len() < right.len() {
        return merge_group_maps(right, left);
    }

    let mut merged = left;
    for (category, mut paths) in right {
        merged.entry(category).or_default().append(&mut paths);
    }
    merged
}

fn group_by_file_type(files: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
    files
        .par_iter()
        .fold(
            || HashMap::<String, Vec<PathBuf>>::new(),
            |mut grouped, path| {
                let category = categorize_path(path);
                grouped.entry(category).or_default().push(path.clone());
                grouped
            },
        )
        .reduce(HashMap::new, merge_group_maps)
}

fn group_by_size(files: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
    let buckets: [(String, f64, f64); 4] = [
        ("Small (< 1MB)".to_string(), 0.0, 1.0),
        ("Medium (1MB - 10MB)".to_string(), 1.0, 10.0),
        ("Large (10MB - 100MB)".to_string(), 10.0, 100.0),
        ("Very Large (> 100MB)".to_string(), 100.0, f64::MAX),
    ];

    files
        .par_iter()
        .filter_map(|path| {
            let size_mb = std::fs::metadata(path).ok()?.len() as f64 / ONE_MB;
            let category = buckets
                .iter()
                .find(|(_, min, max)| size_mb >= *min && (size_mb < *max || *max == f64::MAX))
                .map(|(label, _, _)| label.clone())?;
            Some((category, path.clone()))
        })
        .fold(
            || HashMap::<String, Vec<PathBuf>>::new(),
            |mut grouped, (category, path)| {
                grouped.entry(category).or_default().push(path);
                grouped
            },
        )
        .reduce(HashMap::new, merge_group_maps)
}

fn group_by_date(files: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
    files
        .par_iter()
        .filter_map(|path| {
            let modified = std::fs::metadata(path).ok()?.modified().ok()?;
            let datetime: DateTime<Local> = modified.into();
            let key = datetime.format("%Y-%m").to_string();
            Some((key, path.clone()))
        })
        .fold(
            || HashMap::<String, Vec<PathBuf>>::new(),
            |mut grouped, (category, path)| {
                grouped.entry(category).or_default().push(path);
                grouped
            },
        )
        .reduce(HashMap::new, merge_group_maps)
}
