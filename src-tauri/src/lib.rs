mod claude;
mod commands;
mod db;
mod models;

use claude::SessionManager;
use db::Database;
use models::ProjectState;
use std::sync::Mutex;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let database = Database::new(&app_dir).expect("Failed to initialize database");
            app.manage(database);
            app.manage(SessionManager::new());
            app.manage(ProjectState {
                path: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::move_task,
            commands::run_claude_session,
            commands::stop_claude_session,
            commands::send_input,
            commands::set_project_path,
            commands::get_project_path,
            commands::get_git_status,
            commands::get_output_buffer,
            commands::check_path_exists,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
