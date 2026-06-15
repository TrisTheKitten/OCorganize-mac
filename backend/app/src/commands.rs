use std::path::PathBuf;

use tauri::State;

use crate::organize_job::{run_organize, run_preview, run_undo, OrganizeSummary, PreviewResult, UndoSummary};
use crate::state::AppState;
use ocorganize_core::Strategy;

#[tauri::command]
pub fn validate_directory(path: String) -> Result<String, String> {
    ocorganize_core::sanitize_directory_path(&path)
        .map(|resolved| resolved.to_string_lossy().into_owned())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn preview_organization(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
    strategy: String,
) -> Result<PreviewResult, String> {
    let (preview, group_result) = run_preview(app, path.clone(), strategy.clone()).await?;
    let directory = ocorganize_core::sanitize_directory_path(&path)
        .map_err(|error| error.to_string())?;
    let strategy_enum = Strategy::try_from_str(&strategy)?;
    state.store_scan_cache(directory, strategy_enum, group_result)?;
    Ok(preview)
}

#[tauri::command]
pub async fn organize_files(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
    strategy: String,
    preview_only: bool,
    create_undo: bool,
) -> Result<OrganizeSummary, String> {
    let directory = ocorganize_core::sanitize_directory_path(&path)
        .map_err(|error| error.to_string())?;
    let strategy_enum = Strategy::try_from_str(&strategy)?;
    let precomputed_groups = state.take_matching_scan_cache(&directory, strategy_enum)?;

    let run_result = run_organize(
        app,
        path,
        strategy,
        preview_only,
        create_undo,
        precomputed_groups,
    )
    .await?;

    if preview_only {
        let _ = state.store_scan_cache(
            directory,
            strategy_enum,
            run_result.group_snapshot,
        );
    } else if run_result.summary.undo_file.is_some() {
        let _ = state.clear_scan_cache();
    }

    if let Some(undo_file) = &run_result.summary.undo_file {
        state
            .operation_history
            .lock()
            .map_err(|_| "Failed to lock app state".to_string())?
            .push(PathBuf::from(undo_file));
    }
    Ok(run_result.summary)
}

#[tauri::command]
pub fn undo_last_operation(state: State<'_, AppState>) -> Result<UndoSummary, String> {
    let undo_file = state
        .operation_history
        .lock()
        .map_err(|_| "Failed to lock app state".to_string())?
        .pop()
        .ok_or_else(|| "No operations to undo.".to_string())?;
    let summary = run_undo(undo_file.to_string_lossy().into_owned())?;
    Ok(summary)
}

#[tauri::command]
pub fn has_undo_operations(state: State<'_, AppState>) -> Result<bool, String> {
    let history = state
        .operation_history
        .lock()
        .map_err(|_| "Failed to lock app state".to_string())?;
    Ok(!history.is_empty())
}
