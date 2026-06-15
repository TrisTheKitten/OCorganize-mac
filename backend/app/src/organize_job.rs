use std::path::PathBuf;
use std::sync::Arc;

use ocorganize_core::{
    group_files_with_profile, organize_files, restore_undo_log, sanitize_directory_path,
    should_emit_progress, walk_grouped_files, GroupResult, OrganizeOptions, OrganizeProgress,
    RuntimeProfile, Strategy,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

const ORGANIZATION_PROGRESS_START: f64 = 0.1;
const ORGANIZATION_PROGRESS_SPAN: f64 = 0.8;
const ORGANIZATION_PROGRESS_COMPLETE: f64 = 1.0;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewCategory {
    pub category: String,
    pub folder_name: String,
    pub file_count: usize,
    pub sample_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    pub directory: String,
    pub strategy: String,
    pub total_files: usize,
    pub category_count: usize,
    pub symlink_skip_count: usize,
    pub categories: Vec<PreviewCategory>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeSummary {
    pub organized_count: usize,
    pub category_count: usize,
    pub symlink_skip_count: usize,
    pub preview_only: bool,
    pub undo_file: Option<String>,
}

pub struct OrganizeRunResult {
    pub summary: OrganizeSummary,
    pub group_snapshot: GroupResult,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoSummary {
    pub restored_count: usize,
    pub failed_count: usize,
    pub partial: bool,
}

#[derive(Debug, Clone, Serialize)]
struct OrganizeProgressEvent {
    status: String,
    progress: f64,
    file_name: String,
    processed_files: usize,
    total_files: usize,
}

fn preview_from_group_result(
    directory: PathBuf,
    strategy: Strategy,
    result: GroupResult,
) -> PreviewResult {
    if result.grouped.is_empty() {
        return PreviewResult {
            directory: directory.to_string_lossy().into_owned(),
            strategy: strategy.as_str().to_string(),
            total_files: 0,
            category_count: 0,
            symlink_skip_count: result.symlink_skip_count,
            categories: Vec::new(),
        };
    }

    const PREVIEW_SAMPLE_LIMIT: usize = 10;
    let mut categories: Vec<PreviewCategory> = result
        .grouped
        .iter()
        .map(|(category, files)| {
            let sample_files = files
                .iter()
                .take(PREVIEW_SAMPLE_LIMIT)
                .filter_map(|file| {
                    file.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                })
                .collect();
            PreviewCategory {
                category: category.clone(),
                folder_name: ocorganize_core::get_target_folder_name(category, strategy),
                file_count: files.len(),
                sample_files,
            }
        })
        .collect();
    categories.sort_by(|left, right| left.category.cmp(&right.category));

    let total_files = categories.iter().map(|item| item.file_count).sum();

    PreviewResult {
        directory: directory.to_string_lossy().into_owned(),
        strategy: strategy.as_str().to_string(),
        total_files,
        category_count: categories.len(),
        symlink_skip_count: result.symlink_skip_count,
        categories,
    }
}

fn emit_scan_progress(
    app: &AppHandle,
    group_result: &GroupResult,
    profile: &RuntimeProfile,
    status_prefix: &str,
) {
    let total_files = group_result.total_files();
    if total_files == 0 {
        return;
    }

    let progress_interval = profile.progress_interval_for_total(total_files);
    let mut processed_files = 0usize;
    let app = app.clone();
    let status_prefix = status_prefix.to_string();

    walk_grouped_files(&group_result.grouped, |category, file_path| {
        processed_files += 1;
        if !should_emit_progress(processed_files, total_files, progress_interval) {
            return;
        }

        let ratio = processed_files as f64 / total_files as f64;
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        let _ = app.emit(
            "organize-progress",
            OrganizeProgressEvent {
                status: format!("{status_prefix} {category}..."),
                progress: ORGANIZATION_PROGRESS_START * ratio,
                file_name,
                processed_files,
                total_files,
            },
        );
    });
}

pub async fn run_preview(
    app: AppHandle,
    path: String,
    strategy: String,
) -> Result<(PreviewResult, GroupResult), String> {
    let directory = sanitize_directory_path(&path).map_err(|error| error.to_string())?;
    let strategy_enum = Strategy::try_from_str(&strategy)?;
    let profile = RuntimeProfile::global().clone();
    let app_for_scan = app.clone();

    let (directory, result) = tauri::async_runtime::spawn_blocking(move || {
        let result = group_files_with_profile(&directory, strategy_enum, &profile);
        emit_scan_progress(&app_for_scan, &result, &profile, "Scanning");
        (directory, result)
    })
    .await
    .map_err(|error| error.to_string())?;

    let preview = preview_from_group_result(directory.clone(), strategy_enum, result.clone());
    Ok((preview, result))
}

pub async fn run_organize(
    app: AppHandle,
    path: String,
    strategy: String,
    preview_only: bool,
    create_undo: bool,
    precomputed_groups: Option<GroupResult>,
) -> Result<OrganizeRunResult, String> {
    let directory = sanitize_directory_path(&path).map_err(|error| error.to_string())?;
    let strategy_enum = Strategy::try_from_str(&strategy)?;
    let profile = RuntimeProfile::global().clone();

    if preview_only {
        let group_snapshot = match precomputed_groups {
            Some(groups) => groups,
            None => {
                let app_for_scan = app.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    let result = group_files_with_profile(&directory, strategy_enum, &profile);
                    emit_scan_progress(&app_for_scan, &result, &profile, "Scanning");
                    result
                })
                .await
                .map_err(|error| error.to_string())?
            }
        };

        let _ = app.emit(
            "organize-progress",
            OrganizeProgressEvent {
                status: "Preview complete".to_string(),
                progress: ORGANIZATION_PROGRESS_COMPLETE,
                file_name: String::new(),
                processed_files: group_snapshot.total_files(),
                total_files: group_snapshot.total_files(),
            },
        );

        return Ok(OrganizeRunResult {
            summary: OrganizeSummary {
                organized_count: group_snapshot.total_files(),
                category_count: group_snapshot.grouped.len(),
                symlink_skip_count: group_snapshot.symlink_skip_count,
                preview_only: true,
                undo_file: None,
            },
            group_snapshot,
        });
    }

    let app_for_progress = app.clone();
    let progress_callback: Arc<dyn Fn(OrganizeProgress<'_>) + Send + Sync> =
        Arc::new(move |progress: OrganizeProgress<'_>| {
            let ratio = progress.processed_files as f64 / progress.total_files as f64;
            let value = ORGANIZATION_PROGRESS_START + ORGANIZATION_PROGRESS_SPAN * ratio;
            let _ = app_for_progress.emit(
                "organize-progress",
                OrganizeProgressEvent {
                    status: format!("Processing {}...", progress.category),
                    progress: value,
                    file_name: progress.file_name.clone(),
                    processed_files: progress.processed_files,
                    total_files: progress.total_files,
                },
            );
        });

    let result = tauri::async_runtime::spawn_blocking(move || {
        organize_files(OrganizeOptions {
            directory: &directory,
            strategy: strategy_enum,
            create_undo,
            precomputed_groups,
            profile: Some(&profile),
            on_progress: Some(progress_callback),
        })
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;

    let _ = app.emit(
        "organize-progress",
        OrganizeProgressEvent {
            status: format!("Complete! Organized {} files", result.organized_count),
            progress: ORGANIZATION_PROGRESS_COMPLETE,
            file_name: String::new(),
            processed_files: result.organized_count,
            total_files: result.organized_count,
        },
    );

    Ok(OrganizeRunResult {
        summary: OrganizeSummary {
            organized_count: result.organized_count,
            category_count: result.category_count,
            symlink_skip_count: result.symlink_skip_count,
            preview_only: false,
            undo_file: result
                .undo_file
                .map(|path| path.to_string_lossy().into_owned()),
        },
        group_snapshot: result.group_snapshot,
    })
}

pub fn run_undo(undo_file: String) -> Result<UndoSummary, String> {
    let path = PathBuf::from(&undo_file);
    let result = restore_undo_log(&path).map_err(|error| error.to_string())?;
    Ok(UndoSummary {
        restored_count: result.restored_count,
        failed_count: result.failed_count,
        partial: result.partial(),
    })
}
