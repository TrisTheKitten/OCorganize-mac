pub mod error;
pub mod file_type;
pub mod group;
pub mod move_file;
pub mod naming;
pub mod organize;
pub mod path;
pub mod runtime;
pub mod undo;
pub mod walk;

pub use error::{OrganizeError, PathValidationError};
pub use file_type::{categorize_extension, categorize_path, is_known_file_type};
pub use group::{
    group_files, group_files_with_profile, walk_grouped_files, GroupResult, Strategy,
};
pub use move_file::{
    atomic_move_with_unique_name, atomic_move_with_unique_name_locked, TargetDirLocks,
};
pub use naming::{
    ensure_category_output_dir, get_target_folder_name, is_organized_category_dir,
    is_organized_category_folder_name, ORGANIZED_CATEGORY_MARKER,
};
pub use path::{
    is_finder_visible, is_loose_file_at_root, path_inside_organized_category_folder,
};
pub use organize::{
    organize_files, should_emit_progress, OrganizeOptions, OrganizeProgress, OrganizeResult,
};
pub use path::sanitize_directory_path;
pub use runtime::{PerformanceTier, RuntimeProfile};
pub use undo::{
    read_undo_log, restore_undo_log, undo_entry, write_undo_log, UndoEntry, UndoLogWriter,
    UndoRestoreResult,
};
pub use walk::{
    collect_loose_root_files, collect_loose_root_files_with_profile, WalkResult,
};
