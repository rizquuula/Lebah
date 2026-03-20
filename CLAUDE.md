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
make clean   # clean build artifacts
make tree    # print project file tree
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
    │   └── TerminalModal.svelte   # Live terminal output with stdin
    ├── stores/
    │   ├── tasks.ts               # Task CRUD + session management
    │   └── project.ts             # Project path + git status
    ├── actions/
    │   └── portal.ts              # Svelte portal action for modals
    └── types.ts                   # Shared TypeScript interfaces

src-tauri/src/
├── main.rs                        # Tauri entry point
├── lib.rs                         # App setup, command registration
├── commands.rs                    # Tauri IPC command handlers
├── db.rs                          # SQLite schema + CRUD
├── models.rs                      # Shared data structs/enums
└── claude.rs                      # Claude CLI process management
```

## Code Style
- Keep all files under 300 lines; split into smaller modules if a file approaches this limit

## Git
- Do NOT add `Co-Authored-By` lines to commit messages

## Architecture
- Each task has a UUID used as the `--session-id` for Claude Code CLI
- Tasks flow through columns: To-Do → In Progress → Review → Merge → Completed
- Claude sessions are spawned as child processes, output streamed via Tauri events
- Task status reflected by card border color: green=success, yellow=running, red=failed, orange=warning
- Plan mode passes `--permission-mode plan` to the CLI
- Yolo mode passes `--dangerously-skip-permissions` with `IS_SANDBOX=true` env var

## Tauri Commands (IPC)
| Command | Description |
|---|---|
| `get_tasks` | Fetch all tasks from DB |
| `create_task` | Create task with optional claude_path/claude_command |
| `update_task` | Update task fields |
| `delete_task` | Delete task |
| `move_task` | Move task to column with new sort order |
| `run_claude_session` | Spawn Claude process, stream output via events |
| `stop_claude_session` | Terminate running session |
| `send_input` | Write to session stdin |
| `get_output_buffer` | Get accumulated output for a session |
| `set_project_path` | Set active project directory |
| `get_project_path` | Retrieve current project directory |
| `get_git_status` | Query git branch/ahead/behind/changed files |
