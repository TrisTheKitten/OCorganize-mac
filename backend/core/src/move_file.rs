use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::Mutex;

use crate::error::OrganizeError;
use crate::path::is_symlink;

pub struct TargetDirLocks {
    locks: DashMap<PathBuf, Arc<Mutex<()>>>,
}

impl TargetDirLocks {
    pub fn new() -> Self {
        Self {
            locks: DashMap::new(),
        }
    }

    pub fn lock_for(&self, target_dir: &Path) -> Arc<Mutex<()>> {
        self.locks
            .entry(target_dir.to_path_buf())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

impl Default for TargetDirLocks {
    fn default() -> Self {
        Self::new()
    }
}

pub fn atomic_move_with_unique_name(
    source: &Path,
    target_dir: &Path,
) -> Result<PathBuf, OrganizeError> {
    atomic_move_with_unique_name_locked(source, target_dir, None)
}

pub fn atomic_move_with_unique_name_locked(
    source: &Path,
    target_dir: &Path,
    target_lock: Option<&Mutex<()>>,
) -> Result<PathBuf, OrganizeError> {
    if is_symlink(source) {
        return Err(OrganizeError::SymlinkedFile);
    }

    let _guard = target_lock.map(|lock| lock.lock());

    std::fs::create_dir_all(target_dir)?;
    let reserved = reserve_unique_path(target_dir, source)?;
    match std::fs::rename(source, &reserved) {
        Ok(()) => Ok(reserved),
        Err(error) => {
            let _ = std::fs::remove_file(&reserved);
            Err(OrganizeError::Io(error))
        }
    }
}

fn reserve_unique_path(target_dir: &Path, source: &Path) -> Result<PathBuf, OrganizeError> {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let suffix = source
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{s}"))
        .unwrap_or_default();

    let mut counter = 0u32;
    loop {
        let extra = if counter == 0 {
            String::new()
        } else {
            format!("_{counter}")
        };
        let candidate = target_dir.join(format!("{stem}{extra}{suffix}"));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                drop(file);
                return Ok(candidate);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                counter += 1;
            }
            Err(error) => return Err(OrganizeError::Io(error)),
        }
    }
}
