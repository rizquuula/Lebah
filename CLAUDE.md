# Lebah

Claude Code orchestration desktop app — a kanban board for managing Claude Code CLI sessions.

## Tech Stack
- **Backend**: Rust + Tauri v2
- **Frontend**: Svelte 5 + TypeScript
- **Styling**: Tailwind CSS v4
- **Database**: SQLite (rusqlite with bundled feature)
- **Drag-and-drop**: svelte-dnd-action

## Development

```bash
make setup   # install dependencies (Tauri CLI, npm packages, cargo deps)
make dev     # run in development mode
make build   # build for production
make run     # run the production build
make test    # run Rust tests, Svelte type check, and Cargo check
make clean   # clean build artifacts and dependencies
make clean-soft  # clear only caches (Vite, Cargo incremental)
make tree    # print project file tree with line counts
```

## Project Structure

```
src/
├── App.svelte                    # Root component, project selector, git status
├── main.ts                       # Entry point
└── lib/
    ├── components/
    │   ├── Board.svelte           # Kanban board layout
    │   ├── Column.svelte          # Draggable column with drop zones
    │   ├── TaskCard.svelte        # Task card with controls
    │   ├── TaskModal.svelte       # Task creation/editing modal
    │   ├── TaskDetailModal.svelte # Read-only task detail view
    │   ├── TerminalModal.svelte   # Live terminal output with stdin
    │   ├── TerminalChat.svelte    # Claude output parser with tool use visualization
    │   ├── ConfirmDialog.svelte   # Reusable confirmation dialog
    │   └── TaskToggles.svelte     # Plan/Yolo/Auto toggle controls
    ├── stores/
    │   ├── tasks.ts               # Task CRUD + session management + auto-advance logic
    │   ├── project.ts             # Project path + git status
    │   ├── config.ts              # Project configuration (column templates)
    │   └── errors.ts              # Global error state
    ├── actions/
    │   └── portal.ts              # Svelte portal action for modals
    └── types.ts                   # Shared TypeScript interfaces

src-tauri/src/
├── main.rs                        # Tauri entry point
├── lib.rs                         # App setup, DI container, command registration
├── domain/                        # Business entities, aggregates, repository interfaces
│   ├── task/
│   ├── session/
│   ├── project/
│   ├── agent/
│   ├── git/
│   ├── repositories.rs
│   └── errors.rs
├── application/                   # Use cases, services, event bus
│   ├── task/
│   ├── session/
│   ├── project/
│   ├── git/
│   ├── event_bus.rs
│   └── ports.rs
├── infrastructure/                # Concrete implementations
│   ├── agents/claude/             # Claude CLI runner
│   ├── persistence/               # File-based repositories
│   ├── session/                   # Process session manager
│   ├── event_handlers/            # Tauri event emitter, output persistence
│   └── app_services.rs            # Dependency injection container
└── presentation/
    ├── commands/                  # Tauri IPC command handlers
    │   ├── task_commands.rs
    │   ├── session_commands.rs
    │   ├── project_commands.rs
    │   └── utility_commands.rs
    └── dto.rs                     # Data transfer objects
```

## Code Style
- Keep all files under 300 lines; split into smaller modules if a file approaches this limit

## Git
- Do NOT add `Co-Authored-By` lines to commit messages

## Architecture
- Each task has a UUID used as the `--session-id` for Claude Code CLI
- Tasks flow through columns: To-Do → In Progress → Review → Merge → Completed
- Claude sessions are spawned as child processes, output streamed via Tauri events
- Task status reflected by card border color: green=success, yellow=running, red=failed, orange=warning, blue=waiting (merge queue)
- Plan mode passes `--permission-mode plan` to the CLI
- Yolo mode passes `--dangerously-skip-permissions` with `IS_SANDBOX=true` env var
- Backend follows Domain-Driven Design (DDD) with Clean Architecture layers: domain → application → infrastructure → presentation

### Auto-Advance
When the **Auto** flag is enabled on a task:
- InProgress task succeeds → moves to Review, sends `inprogress_template` as input
- Review task succeeds → moves to Merge (or queues as Waiting if another merge is running), sends `review_template` as input
- Merge task succeeds → moves to Completed, starts next queued merge task, sends `merge_template` as input

### Project Templates
Configurable per-project prompt templates stored via `get_project_config` / `set_project_config`:
- `inprogress_template` — injected when auto-advancing from InProgress
- `review_template` — sent as input when auto-advancing to Review
- `merge_template` — sent as input when auto-advancing to Merge

### Model Selection
Tasks support a `model` field to override the Claude model per task. The model can also be overridden when sending input via `send_input`.

## Tauri Commands (IPC)
| Command | Description |
|---|---|
| `get_tasks` | Fetch all tasks from DB |
| `create_task` | Create task with optional claude_path/claude_command/model |
| `update_task` | Update task fields |
| `delete_task` | Delete task |
| `move_task` | Move task to column with new sort order |
| `reset_task_session` | Reset a task's session state |
| `run_claude_session` | Spawn Claude process, stream output via events |
| `stop_claude_session` | Terminate running session |
| `send_input` | Write to session stdin (supports model override) |
| `get_output_buffer` | Get accumulated output for a session |
| `set_project_path` | Set active project directory |
| `get_project_path` | Retrieve current project directory |
| `get_git_status` | Query git branch/ahead/behind/changed files |
| `get_project_config` | Retrieve project configuration (column templates) |
| `set_project_config` | Save project configuration |
| `check_path_exists` | Check if a file or directory path exists |
| `generate_worktree_name` | Generate an AI-powered worktree name from a task description |
