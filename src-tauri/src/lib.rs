mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri::Manager;

use application::event_bus::InProcessEventBus;
use domain::repositories::ProjectRepository;
use application::git::service::GitApplicationService;
use application::project::service::ProjectApplicationService;
use application::session::service::SessionApplicationService;
use application::task::service::TaskApplicationService;
use infrastructure::agents::claude::runner::ClaudeRunner;
use infrastructure::agents::registry::AgentRegistry;
use infrastructure::event_handlers::output_persistence_handler::OutputPersistenceHandler;
use infrastructure::event_handlers::session_status_handler::SessionStatusHandler;
use infrastructure::event_handlers::tauri_event_emitter::TauriEventEmitter;
use infrastructure::git::git_service::GitInfraService;
use infrastructure::persistence::file_output_repository::FileOutputRepository;
use infrastructure::persistence::json_project_repository::JsonProjectRepository;
use infrastructure::persistence::json_task_repository::JsonTaskRepository;
use infrastructure::persistence::path_resolver::PathResolver;
use infrastructure::session::process_session_manager::ProcessSessionManager;
use infrastructure::worktree::worktree_manager::WorktreeManager;
use infrastructure::AppServices;

use presentation::commands::project_commands::*;
use presentation::commands::session_commands::*;
use presentation::commands::task_commands::*;
use presentation::commands::utility_commands::*;

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Lebah starting up");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // --- Infrastructure: path resolver ---
            let resolver = Arc::new(PathResolver::new().expect("Failed to init path resolver"));

            // --- Infrastructure: repositories ---
            let task_repo = Arc::new(JsonTaskRepository::new(Arc::clone(&resolver)));
            let project_repo = Arc::new(JsonProjectRepository::new(Arc::clone(&resolver)));
            let output_repo = Arc::new(FileOutputRepository::new(Arc::clone(&resolver)));

            // --- Infrastructure: ports ---
            let git_port = Arc::new(GitInfraService::new());
            let worktree_port = Arc::new(WorktreeManager::new());

            // --- Shared state: current project path ---
            let current_project: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

            // --- Load last project from global config ---
            {
                let config = project_repo.load_global_config();
                if let Some(ref path) = config.last_project {
                    if std::path::Path::new(path.as_str() as &str).is_dir() {
                        *current_project.lock().expect("mutex") = Some(path.clone());
                    }
                }
            }

            // --- Infrastructure: event bus ---
            let event_bus: Arc<dyn application::event_bus::DomainEventBus> =
                Arc::new(InProcessEventBus::new());

            // --- Infrastructure: output buffer (shared between runner + session manager) ---
            let output_buffers: Arc<Mutex<HashMap<String, Vec<String>>>> =
                Arc::new(Mutex::new(HashMap::new()));

            // --- Infrastructure: session manager ---
            let session_manager = Arc::new(ProcessSessionManager::new(Arc::clone(&output_buffers)));

            // --- Infrastructure: agent registry ---
            let mut registry = AgentRegistry::new();
            registry.register(Arc::new(ClaudeRunner::new()));
            let agent_registry = Arc::new(registry);

            // --- Application services ---
            let task_service = Arc::new(TaskApplicationService::new(
                Arc::clone(&task_repo) as Arc<dyn crate::domain::repositories::TaskRepository>,
                Arc::clone(&output_repo) as Arc<dyn crate::domain::repositories::OutputRepository>,
                Arc::clone(&worktree_port) as Arc<dyn crate::application::ports::WorktreePort>,
                Arc::clone(&git_port) as Arc<dyn crate::application::ports::GitPort>,
                Arc::clone(&event_bus),
                Arc::clone(&current_project),
            ));

            let project_service = Arc::new(ProjectApplicationService::new(
                Arc::clone(&project_repo) as Arc<dyn crate::domain::repositories::ProjectRepository>,
                Arc::clone(&current_project),
            ));

            let git_service = Arc::new(GitApplicationService::new(
                Arc::clone(&git_port) as Arc<dyn crate::application::ports::GitPort>,
            ));

            let session_service = Arc::new(SessionApplicationService::new(
                Arc::clone(&agent_registry),
                Arc::clone(&task_service),
                Arc::clone(&output_repo) as Arc<dyn crate::domain::repositories::OutputRepository>,
                Arc::clone(&session_manager) as Arc<dyn crate::application::ports::SessionManagerPort>,
                Arc::clone(&event_bus),
                Arc::clone(&current_project),
            ));

            // --- Event handlers ---
            let tauri_emitter = Arc::new(TauriEventEmitter::new(app.handle().clone()));
            let output_handler = Arc::new(OutputPersistenceHandler::new(
                Arc::clone(&output_repo) as Arc<dyn crate::domain::repositories::OutputRepository>,
                Arc::clone(&output_buffers),
            ));
            let status_handler = Arc::new(SessionStatusHandler::new(Arc::clone(&task_service)));

            event_bus.subscribe(tauri_emitter);
            event_bus.subscribe(output_handler);
            event_bus.subscribe(status_handler);

            // --- Manage composed AppServices ---
            app.manage(AppServices {
                task_service,
                session_service,
                project_service,
                git_service,
                worktree_port: Arc::clone(&worktree_port) as Arc<dyn crate::application::ports::WorktreePort>,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_tasks,
            create_task,
            update_task,
            delete_task,
            move_task,
            run_claude_session,
            stop_claude_session,
            send_input,
            set_project_path,
            get_project_path,
            get_recent_projects,
            get_git_status,
            git_push,
            get_output_buffer,
            check_path_exists,
            generate_worktree_name,
            get_app_version,
            reset_task_session,
            get_project_config,
            set_project_config,
            apply_worktree_links,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
