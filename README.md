# LebahTempa

> A kanban board desktop app for orchestrating multiple Claude Code CLI sessions.

Manage AI coding sessions visually — create tasks, drag them through workflow stages, and run Claude Code with live terminal output, all from a single desktop window.

---

## Features

- **Kanban workflow** — Five columns: To-Do → In Progress → Review → Merge → Completed
- **Drag & drop** — Reorder and move tasks between columns
- **Live terminal** — Fullscreen terminal modal with real-time stdout/stderr and stdin support
- **Session control** — Start and stop Claude Code sessions per task
- **Plan mode** — Run Claude in `--permission-mode plan` (read-only proposals)
- **Yolo mode** — Run with `--dangerously-skip-permissions` for unattended automation
- **Git status** — See current branch, ahead/behind counts, and changed file count
- **Custom Claude path** — Override the Claude CLI binary per task
- **Persistent state** — All tasks and settings saved in a local SQLite database

## Screenshots

> _Coming soon_

## Requirements

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) v18+
- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and on `PATH`
- Linux / macOS / Windows with [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

## Getting Started

```bash
# 1. Clone the repo
git clone https://github.com/your-username/LebahTempa.git
cd LebahTempa

# 2. Install all dependencies
make setup

# 3. Start the development app
make dev
```

## Build

```bash
make build        # production build (outputs to src-tauri/target/release/)
make clean        # remove all build artifacts
make clean-soft   # remove frontend build only
```

## Usage

1. **Set your project** — Click the folder icon to select the directory Claude should work in.
2. **Create a task** — Click **+ New Task**, describe what you want Claude to do, and optionally set a custom Claude path or extra CLI flags.
3. **Run a session** — Press the play button on a task card. Claude Code starts with the task's UUID as `--session-id`.
4. **Watch the terminal** — Click the terminal icon to open the live output modal. Type to send stdin.
5. **Move tasks** — Drag cards across columns as work progresses, or let status changes guide you.

### Task Options

| Option | Description |
|---|---|
| **Plan mode** | Passes `--permission-mode plan`; Claude proposes changes without writing files |
| **Yolo mode** | Passes `--dangerously-skip-permissions`; skips all permission prompts |
| **Claude path** | Override which `claude` binary to use for this task |
| **Claude command** | Append extra CLI arguments (e.g. `--model claude-opus-4-5`) |

### Status Colors

| Color | Meaning |
|---|---|
| Yellow border | Session is running |
| Green border | Session completed successfully |
| Red border | Session exited with error |
| Orange border | Session exited with warning |

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop runtime | [Tauri v2](https://v2.tauri.app/) |
| Backend language | Rust |
| Database | SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) |
| Frontend framework | [Svelte 5](https://svelte.dev/) + TypeScript |
| Styling | [Tailwind CSS v4](https://tailwindcss.com/) |
| Drag & drop | [svelte-dnd-action](https://github.com/isaacs/node-graceful-fs) |

## Project Structure

```
src/                        # Svelte frontend
├── App.svelte              # Root component
└── lib/
    ├── components/         # UI components (Board, Column, TaskCard, Modals)
    ├── stores/             # Reactive state (tasks, project)
    ├── actions/            # Svelte actions (portal)
    └── types.ts            # Shared TypeScript types

src-tauri/src/              # Rust backend
├── commands.rs             # Tauri IPC handlers
├── db.rs                   # SQLite schema & queries
├── models.rs               # Shared data models
└── claude.rs               # Claude CLI process management
```

## Contributing

Contributions are welcome. Please open an issue before submitting a pull request for significant changes.

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/your-feature`)
3. Commit your changes
4. Open a pull request

## License

MIT
