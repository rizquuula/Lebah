# LebahTempa

Claude Code orchestration desktop app — a kanban board for managing Claude Code CLI sessions.

## Tech Stack
- **Backend**: Rust + Tauri v2
- **Frontend**: Svelte 5 + TypeScript
- **Database**: SQLite (rusqlite with bundled feature)
- **Drag-and-drop**: svelte-dnd-action

## Development

```bash
make setup   # install dependencies (Tauri CLI, npm packages, cargo deps)
make dev     # run in development mode
make build   # build for production
make clean   # clean build artifacts
```

## Project Structure
- `src-tauri/src/` — Rust backend (Tauri commands, SQLite, Claude CLI management)
- `src/` — Svelte frontend (kanban board UI)

## Git
- Do NOT add `Co-Authored-By` lines to commit messages

## Architecture
- Each task has a UUID used as the `--session-id` for Claude Code CLI
- Tasks flow through columns: To-Do → In Progress → Review → Merge → Completed
- Claude sessions are spawned as child processes, output streamed via Tauri events
- Task status reflected by card border color: green=success, yellow=running, red=failed, orange=warning
