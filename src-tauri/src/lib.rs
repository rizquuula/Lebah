mod claude;
mod commands;
mod models;
mod storage;

use claude::SessionManager;
use storage::Storage;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let storage = Storage::new().expect("Failed to initialize storage");
            app.manage(storage);
            app.manage(SessionManager::new());
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
            commands::reset_task_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
