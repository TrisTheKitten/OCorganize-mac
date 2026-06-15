mod commands;
mod organize_job;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::validate_directory,
            commands::preview_organization,
            commands::organize_files,
            commands::undo_last_operation,
            commands::has_undo_operations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OCorganize");
}
