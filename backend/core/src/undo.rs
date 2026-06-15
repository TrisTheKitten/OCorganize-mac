use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::Local;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::error::OrganizeError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UndoEntry {
    pub from: String,
    pub to: String,
    pub timestamp: String,
}

pub struct UndoRestoreResult {
    pub restored_count: usize,
    pub failed_count: usize,
}

impl UndoRestoreResult {
    pub fn partial(&self) -> bool {
        self.failed_count > 0
    }
}

pub struct UndoLogWriter {
    writer: Mutex<BufWriter<File>>,
    pending: Mutex<Vec<UndoEntry>>,
    flush_batch: usize,
    path: PathBuf,
    entry_count: Mutex<usize>,
    closed: Mutex<bool>,
}

impl UndoLogWriter {
    pub fn new(directory: &Path, flush_batch: usize) -> Result<Self, OrganizeError> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let path = directory.join(format!("undo_file_organization_{timestamp}.json"));
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(b"[\n")?;

        Ok(Self {
            writer: Mutex::new(writer),
            pending: Mutex::new(Vec::new()),
            flush_batch,
            path,
            entry_count: Mutex::new(0),
            closed: Mutex::new(false),
        })
    }

    pub fn push(&self, entry: UndoEntry) -> Result<(), OrganizeError> {
        let mut pending = self.pending.lock();
        pending.push(entry);
        if pending.len() >= self.flush_batch {
            self.flush_pending(&mut pending)?;
        }
        Ok(())
    }

    pub fn finish(self, mut extra_entries: Vec<UndoEntry>) -> Result<PathBuf, OrganizeError> {
        {
            let mut pending = self.pending.lock();
            pending.append(&mut extra_entries);
            pending.sort_by(|left, right| left.to.cmp(&right.to));
            self.flush_pending(&mut pending)?;
        }

        let path = self.path.clone();
        {
            let mut writer = self.writer.lock();
            writer.write_all(b"\n]\n")?;
            writer.flush()?;
        }
        *self.closed.lock() = true;
        Ok(path)
    }

    fn flush_pending(&self, pending: &mut Vec<UndoEntry>) -> Result<(), OrganizeError> {
        if pending.is_empty() {
            return Ok(());
        }

        let mut writer = self.writer.lock();
        let mut entry_count = self.entry_count.lock();

        for entry in pending.drain(..) {
            if *entry_count > 0 {
                writer.write_all(b",\n")?;
            }
            serde_json::to_writer(&mut *writer, &entry)?;
            *entry_count += 1;
        }

        writer.flush()?;
        Ok(())
    }
}

impl Drop for UndoLogWriter {
    fn drop(&mut self) {
        if *self.closed.lock() {
            return;
        }

        let _ = (|| {
            let mut pending = self.pending.lock();
            self.flush_pending(&mut pending)?;
            let mut writer = self.writer.lock();
            writer.write_all(b"\n]\n")?;
            writer.flush()?;
            *self.closed.lock() = true;
            Ok::<(), OrganizeError>(())
        })();
    }
}

pub fn write_undo_log(directory: &Path, entries: &[UndoEntry]) -> Result<PathBuf, OrganizeError> {
    let mut sorted_entries = entries.to_vec();
    sorted_entries.sort_by(|left, right| left.to.cmp(&right.to));

    let writer = UndoLogWriter::new(directory, sorted_entries.len().max(1))?;
    for entry in sorted_entries {
        writer.push(entry)?;
    }
    writer.finish(Vec::new())
}

pub fn read_undo_log(path: &Path) -> Result<Vec<UndoEntry>, OrganizeError> {
    let content = std::fs::read_to_string(path)?;
    let entries = serde_json::from_str(&content)?;
    Ok(entries)
}

pub fn restore_undo_log(path: &Path) -> Result<UndoRestoreResult, OrganizeError> {
    let entries = read_undo_log(path)?;
    let mut restored_count = 0usize;
    let mut failed_count = 0usize;

    for entry in &entries {
        let current_path = PathBuf::from(&entry.from);
        let original_path = PathBuf::from(&entry.to);
        if let Some(parent) = original_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::rename(&current_path, &original_path) {
            Ok(()) => restored_count += 1,
            Err(_) => failed_count += 1,
        }
    }

    if failed_count == 0 {
        let _ = std::fs::remove_file(path);
    }

    Ok(UndoRestoreResult {
        restored_count,
        failed_count,
    })
}

pub fn undo_entry(current_path: &Path, original_path: &Path) -> UndoEntry {
    UndoEntry {
        from: current_path.to_string_lossy().into_owned(),
        to: original_path.to_string_lossy().into_owned(),
        timestamp: Local::now().to_rfc3339(),
    }
}
