use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use jwalk::{Parallelism, WalkDir};

use crate::naming::{is_organized_category_dir, is_organized_category_folder_name};
use crate::path::{
    is_finder_visible, is_loose_file_at_root, is_symlink, path_inside_organized_category_folder,
};
use crate::runtime::RuntimeProfile;

const INITIAL_FILE_CAPACITY: usize = 256;
const ROOT_CHILD_MAX_DEPTH: usize = 1;

pub struct WalkResult {
    pub files: Vec<PathBuf>,
    pub symlink_skip_count: usize,
    pub organized_folder_skip_count: usize,
    pub hidden_skip_count: usize,
}

pub fn collect_loose_root_files(root: &Path) -> WalkResult {
    collect_loose_root_files_with_profile(root, RuntimeProfile::global())
}

pub fn collect_loose_root_files_with_profile(root: &Path, profile: &RuntimeProfile) -> WalkResult {
    let mut files = Vec::with_capacity(INITIAL_FILE_CAPACITY);
    let mut symlink_skip_count = 0;
    let mut organized_folder_skip_count = 0;
    let mut hidden_skip_count = 0;

    let walker = WalkDir::new(root)
        .follow_links(false)
        .skip_hidden(true)
        .max_depth(ROOT_CHILD_MAX_DEPTH)
        .parallelism(Parallelism::RayonNewPool(profile.walk_threads))
        .process_read_dir(|_depth, parent, _read_dir_state, children| {
            for entry_result in children.iter_mut().flatten() {
                let child_path = parent.join(&entry_result.file_name);
                if should_prune_category_dir(&child_path, &entry_result.file_name) {
                    entry_result.read_children_path = None;
                }
            }
        });

    for entry in walker.into_iter().filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if path == root {
            continue;
        }

        if is_symlink(&path) {
            symlink_skip_count += 1;
            continue;
        }

        if entry.file_type().is_dir() {
            continue;
        }

        if !entry.file_type().is_file() {
            continue;
        }

        if !is_loose_file_at_root(&path, root) {
            continue;
        }

        if path_inside_organized_category_folder(&path, root) {
            organized_folder_skip_count += 1;
            continue;
        }

        if !is_finder_visible(&path, root) {
            hidden_skip_count += 1;
            continue;
        }

        files.push(path.to_path_buf());
    }

    WalkResult {
        files,
        symlink_skip_count,
        organized_folder_skip_count,
        hidden_skip_count,
    }
}

fn should_prune_category_dir(path: &Path, file_name: &OsStr) -> bool {
    file_name
        .to_str()
        .is_some_and(is_organized_category_folder_name)
        || is_organized_category_dir(path)
}
