use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

use crate::error::OrganizeError;
use crate::group::{group_files_with_profile, GroupResult, Strategy};
use crate::move_file::{
    atomic_move_with_unique_name, atomic_move_with_unique_name_locked, TargetDirLocks,
};
use crate::naming::{ensure_category_output_dir, get_target_folder_name};
use crate::runtime::RuntimeProfile;
use crate::undo::{undo_entry, UndoLogWriter};

pub struct OrganizeProgress<'a> {
    pub category: &'a str,
    pub file_name: String,
    pub processed_files: usize,
    pub total_files: usize,
}

pub struct OrganizeOptions<'a> {
    pub directory: &'a Path,
    pub strategy: Strategy,
    pub create_undo: bool,
    pub precomputed_groups: Option<GroupResult>,
    pub profile: Option<&'a RuntimeProfile>,
    pub on_progress: Option<Arc<dyn Fn(OrganizeProgress<'_>) + Send + Sync>>,
}

pub struct OrganizeResult {
    pub organized_count: usize,
    pub category_count: usize,
    pub symlink_skip_count: usize,
    pub undo_file: Option<PathBuf>,
    pub group_snapshot: GroupResult,
}

struct OrganizeExecution {
    organized_count: usize,
    category_count: usize,
    symlink_skip_count: usize,
    undo_file: Option<PathBuf>,
}

struct CategoryMovePlan {
    category: String,
    target_dir: PathBuf,
    files: Vec<PathBuf>,
}

struct MoveTask {
    category: String,
    file_path: PathBuf,
    target_dir: PathBuf,
}

pub fn should_emit_progress(processed_files: usize, total_files: usize, progress_interval: usize) -> bool {
    processed_files % progress_interval == 0 || processed_files == total_files
}

pub fn organize_files(options: OrganizeOptions<'_>) -> Result<OrganizeResult, OrganizeError> {
    let profile = options
        .profile
        .cloned()
        .unwrap_or_else(|| RuntimeProfile::global().clone());
    let directory = options.directory;
    let strategy = options.strategy;
    let create_undo = options.create_undo;
    let on_progress = options.on_progress;
    let group_result = match options.precomputed_groups {
        Some(result) => result,
        None => group_files_with_profile(directory, strategy, &profile),
    };
    let group_snapshot = group_result.clone();
    let total_files = group_result.total_files();

    if total_files == 0 {
        return Ok(OrganizeResult {
            organized_count: 0,
            category_count: 0,
            symlink_skip_count: group_result.symlink_skip_count,
            undo_file: None,
            group_snapshot,
        });
    }

    let progress_interval = profile.progress_interval_for_total(total_files);
    let mut categories: Vec<_> = group_result.grouped.into_iter().collect();
    categories.sort_by(|left, right| left.0.cmp(&right.0));
    let category_count = categories.len();
    let plans = build_move_plans(directory, strategy, categories)?;

    let execution = if profile.move_concurrency <= 1 {
        run_sequential_moves(
            create_undo,
            on_progress.as_ref(),
            plans,
            directory,
            total_files,
            progress_interval,
            group_result.symlink_skip_count,
            category_count,
            &profile,
        )?
    } else {
        run_parallel_moves(
            create_undo,
            on_progress.as_ref(),
            plans,
            directory,
            total_files,
            progress_interval,
            group_result.symlink_skip_count,
            category_count,
            &profile,
        )?
    };

    Ok(OrganizeResult {
        organized_count: execution.organized_count,
        category_count: execution.category_count,
        symlink_skip_count: execution.symlink_skip_count,
        undo_file: execution.undo_file,
        group_snapshot,
    })
}

fn build_move_plans(
    directory: &Path,
    strategy: Strategy,
    categories: Vec<(String, Vec<PathBuf>)>,
) -> Result<Vec<CategoryMovePlan>, OrganizeError> {
    let mut plans = Vec::with_capacity(categories.len());
    for (category, mut files) in categories {
        let folder_name = get_target_folder_name(&category, strategy);
        let target_dir = directory.join(&folder_name);
        ensure_category_output_dir(&target_dir).map_err(OrganizeError::Io)?;
        files.sort();
        plans.push(CategoryMovePlan {
            category,
            target_dir,
            files,
        });
    }
    Ok(plans)
}

fn run_sequential_moves(
    create_undo: bool,
    on_progress: Option<&Arc<dyn Fn(OrganizeProgress<'_>) + Send + Sync>>,
    plans: Vec<CategoryMovePlan>,
    directory: &Path,
    total_files: usize,
    progress_interval: usize,
    symlink_skip_count: usize,
    category_count: usize,
    profile: &RuntimeProfile,
) -> Result<OrganizeExecution, OrganizeError> {
    let undo_writer = if create_undo {
        Some(UndoLogWriter::new(directory, profile.undo_flush_batch)?)
    } else {
        None
    };
    let mut processed_files = 0usize;
    let mut organized_count = 0usize;

    for plan in plans {
        for file_path in &plan.files {
            processed_files += 1;
            let target_path = atomic_move_with_unique_name(file_path, &plan.target_dir)?;
            if let Some(writer) = &undo_writer {
                writer.push(undo_entry(&target_path, file_path))?;
            }
            organized_count += 1;
            emit_progress(
                on_progress,
                &plan.category,
                file_path,
                processed_files,
                total_files,
                progress_interval,
            );
        }
    }

    let undo_file = if create_undo {
        Some(
            undo_writer
                .expect("undo writer must exist when create_undo is enabled")
                .finish(Vec::new())?,
        )
    } else {
        None
    };

    Ok(OrganizeExecution {
        organized_count,
        category_count,
        symlink_skip_count,
        undo_file,
    })
}

fn run_parallel_moves(
    create_undo: bool,
    on_progress: Option<&Arc<dyn Fn(OrganizeProgress<'_>) + Send + Sync>>,
    plans: Vec<CategoryMovePlan>,
    directory: &Path,
    total_files: usize,
    progress_interval: usize,
    symlink_skip_count: usize,
    category_count: usize,
    profile: &RuntimeProfile,
) -> Result<OrganizeExecution, OrganizeError> {
    let mut tasks = Vec::with_capacity(total_files);
    for plan in plans {
        for file_path in plan.files {
            tasks.push(MoveTask {
                category: plan.category.clone(),
                file_path,
                target_dir: plan.target_dir.clone(),
            });
        }
    }

    let pool = ThreadPoolBuilder::new()
        .num_threads(profile.move_concurrency)
        .build()
        .map_err(|error| OrganizeError::Io(std::io::Error::other(error.to_string())))?;

    let target_locks = Arc::new(TargetDirLocks::new());
    let undo_writer = if create_undo {
        Some(Arc::new(UndoLogWriter::new(
            directory,
            profile.undo_flush_batch,
        )?))
    } else {
        None
    };
    let processed_files = AtomicUsize::new(0);
    let organized_count = AtomicUsize::new(0);

    pool.install(|| {
        tasks.par_iter().try_for_each(|task| -> Result<(), OrganizeError> {
            let target_lock = target_locks.lock_for(&task.target_dir);
            let target_path = atomic_move_with_unique_name_locked(
                &task.file_path,
                &task.target_dir,
                Some(target_lock.as_ref()),
            )?;

            if let Some(writer) = &undo_writer {
                writer.push(undo_entry(&target_path, &task.file_path))?;
            }

            let processed = processed_files.fetch_add(1, Ordering::Relaxed) + 1;
            organized_count.fetch_add(1, Ordering::Relaxed);
            emit_progress(
                on_progress,
                &task.category,
                &task.file_path,
                processed,
                total_files,
                progress_interval,
            );
            Ok(())
        })
    })?;

    let undo_file = if let Some(writer) = undo_writer {
        Some(
            Arc::try_unwrap(writer)
                .map_err(|_| OrganizeError::Io(std::io::Error::other("undo writer still shared")))?
                .finish(Vec::new())?,
        )
    } else {
        None
    };

    Ok(OrganizeExecution {
        organized_count: organized_count.load(Ordering::Relaxed),
        category_count,
        symlink_skip_count,
        undo_file,
    })
}

fn emit_progress(
    callback: Option<&Arc<dyn Fn(OrganizeProgress<'_>) + Send + Sync>>,
    category: &str,
    file_path: &Path,
    processed_files: usize,
    total_files: usize,
    progress_interval: usize,
) {
    if !should_emit_progress(processed_files, total_files, progress_interval) {
        return;
    }

    if let Some(callback) = callback {
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        callback(OrganizeProgress {
            category,
            file_name,
            processed_files,
            total_files,
        });
    }
}
