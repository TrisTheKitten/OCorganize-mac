use std::path::{Path, PathBuf};

use crate::error::PathValidationError;
use crate::naming::is_organized_category_dir;

#[cfg(target_os = "macos")]
const UF_HIDDEN: u32 = 0x0000_8000;

pub fn sanitize_directory_path(raw_path: &str) -> Result<PathBuf, PathValidationError> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Err(PathValidationError::Empty);
    }
    if trimmed.contains('\0') {
        return Err(PathValidationError::InvalidPath);
    }

    let expanded = expand_home(trimmed);
    if expanded.is_symlink() {
        return Err(PathValidationError::SymlinkedDirectory);
    }

    let resolved = expanded.canonicalize().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PathValidationError::NotFound
        } else {
            PathValidationError::InvalidDirectory(error.to_string())
        }
    })?;

    if !resolved.is_dir() {
        return Err(PathValidationError::NotADirectory);
    }

    let metadata = std::fs::metadata(&resolved).map_err(|error| {
        PathValidationError::InvalidDirectory(error.to_string())
    })?;
    let permissions = metadata.permissions();
    if permissions.readonly() {
        return Err(PathValidationError::NotAccessible);
    }

    Ok(resolved)
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs_home() {
            return home.join(rest);
        }
    }
    if path == "~" {
        if let Some(home) = dirs_home() {
            return home;
        }
    }
    PathBuf::from(path)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
}

pub fn is_finder_visible(path: &Path, root: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    if relative.components().any(|component| {
        if let std::path::Component::Normal(name) = component {
            name.to_str().is_some_and(|value| value.starts_with('.'))
        } else {
            false
        }
    }) {
        return false;
    }

    #[cfg(target_os = "macos")]
    {
        use std::os::darwin::fs::MetadataExt;

        if let Ok(metadata) = path.symlink_metadata() {
            if metadata.st_flags() & UF_HIDDEN != 0 {
                return false;
            }
        }
    }

    true
}

pub fn is_loose_file_at_root(path: &Path, root: &Path) -> bool {
    path.parent() == Some(root)
}

pub fn path_inside_organized_category_folder(path: &Path, root: &Path) -> bool {
    let Some(mut current) = path.parent().map(Path::to_path_buf) else {
        return false;
    };

    while current.starts_with(root) && current != root {
        if is_organized_category_dir(&current) {
            return true;
        }
        if !current.pop() {
            break;
        }
    }

    false
}
