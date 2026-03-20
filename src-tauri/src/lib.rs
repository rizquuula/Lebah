mod claude;
mod commands;
mod db;
mod models;

use claude::SessionManager;
use db::Database;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let database = Database::new(&app_dir).expect("Failed to initialize database");
            app.manage(database);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
