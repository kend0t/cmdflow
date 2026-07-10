# ⚡ CmdFlow

**Save multi-command workflows as shortcuts and run them in a few keystrokes.**

CmdFlow is a terminal workflow manager built in Rust. If you find yourself repeatedly typing the same series of commands — switching environments, deploying code, setting up dev servers — CmdFlow lets you save those steps once and replay them anytime with a single command.

---

## ✨ Features

- **Save workflows** — Group multiple commands under a single named shortcut
- **Interactive TUI** — Full terminal UI for creating, browsing, and running workflows
- **Shell history browser** — Press `↑` to browse and pick from your actual shell history instead of retyping commands
- **Fuzzy search** — Type to filter through your shell history instantly
- **Cross-platform** — Works on Windows (PowerShell), macOS (zsh/bash), and Linux
- **Sequential execution** — Runs commands in order with live output and colored status indicators
- **Stop on failure** — Halts on errors by default, with a `--force` flag to power through
- **Portable storage** — Workflows are saved as a single JSON file, accessible from any directory

---

## 📦 Installation

### Prerequisites

- [Rust](https://rustup.rs/) (1.70 or later)

### Install from source

```bash
# Clone or download the project
git clone https://github.com/your-username/cmdflow.git
cd cmdflow

# Install globally
cargo install --path .
```

This places `cmdflow` in `~/.cargo/bin/`, which is added to your PATH by rustup. You may need to restart your terminal for the PATH to take effect.

### Verify installation

```bash
cmdflow --version
```

> **Note:** If your terminal doesn't recognize `cmdflow` after installing, restart your terminal or refresh your PATH:
> - **Windows (PowerShell):**
>   ```powershell
>   $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
>   ```
> - **macOS/Linux:**
>   ```bash
>   source ~/.bashrc   # or ~/.zshrc
>   ```

---

## 🚀 Quick Start

### 1. Create a workflow

```bash
cmdflow new
```

This opens an interactive TUI where you can:
- Name your workflow (e.g., `deploy-qa`)
- Add a description
- Add commands by typing them or browsing your shell history with `↑`

### 2. Run a workflow

```bash
# Interactive selector with preview
cmdflow run

# Or run directly by name
cmdflow run deploy-qa
```

### 3. That's it!

Your workflow runs each command in sequence, showing live output with colored status:

```
  ▶ Running 3 command(s)...

  [1/3] gcloud config set project my-project-qa
       Updated property [core/project].
  [1/3] ✓ Done

  [2/3] gcloud builds submit --tag gcr.io/my-project-qa/my-app
       ...
  [2/3] ✓ Done

  [3/3] gcloud run deploy my-app --image gcr.io/my-project-qa/my-app
       Service [my-app] revision [my-app-00042] has been deployed
  [3/3] ✓ Done

  ✅ All 3 command(s) completed successfully!
```

---

## 📖 Commands

| Command | Description |
|---------|-------------|
| `cmdflow new` | Create a new workflow interactively |
| `cmdflow run` | Select and run a workflow (interactive picker) |
| `cmdflow run <name>` | Run a specific workflow by name |
| `cmdflow run <name> --force` | Continue running even if a command fails |
| `cmdflow list` | List all saved workflows |
| `cmdflow show <name>` | View the details and commands of a workflow |
| `cmdflow edit <name>` | Edit an existing workflow |
| `cmdflow delete` | Interactively select a workflow to delete |
| `cmdflow delete <name>` | Delete a workflow by name |

Running `cmdflow` with no arguments is the same as `cmdflow run` — it opens the interactive workflow picker.

---

## ⌨️ Keyboard Shortcuts

### Create / Edit Screen

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between Name, Description, and Command fields |
| `Enter` | Start typing in a field / Add a typed command |
| `↑` | Open shell history browser (on the Command field) |
| `j` / `k` | Navigate up/down in the command list |
| `Shift+J` / `Shift+K` | Reorder commands (move down/up) |
| `Del` or `d` | Remove the selected command |
| `Ctrl+S` | Save the workflow |
| `Esc` | Cancel or stop editing a field |

### History Browser

| Key | Action |
|-----|--------|
| Type anything | Filter history by keyword |
| `↑` / `↓` | Navigate through matches |
| `Enter` | Select and add the command |
| `Backspace` | Edit the filter text |
| `Esc` | Close the history browser |

### Run Selector / Delete Selector

| Key | Action |
|-----|--------|
| `↑` / `↓` or `k` / `j` | Navigate workflows |
| `Enter` | Run or confirm selection |
| `Esc` or `q` | Cancel |

---

## 💾 Storage

Workflows are stored as a single JSON file:

| OS | Location |
|----|----------|
| **Windows** | `%APPDATA%\cmdflow\workflows.json` |
| **macOS** | `~/Library/Application Support/cmdflow/workflows.json` |
| **Linux** | `~/.config/cmdflow/workflows.json` |

The file is human-readable and can be manually edited or backed up.

---

## 🔧 Shell History Support

CmdFlow reads your shell history so you can pick previously-used commands instead of retyping them. It automatically detects and reads from:

| Shell | History File |
|-------|-------------|
| **PowerShell** (Windows) | `%APPDATA%\Microsoft\Windows\PowerShell\PSReadline\ConsoleHost_history.txt` |
| **zsh** (macOS default) | `~/.zsh_history` |
| **bash** | `~/.bash_history` |
| **fish** | `~/.local/share/fish/fish_history` |

---

## 🗂️ Use Cases

- **Deploy pipelines** — Switch cloud project, build, and deploy in one go
- **Dev environment setup** — Start databases, run migrations, launch servers
- **Git workflows** — Stash, pull, rebase, pop in a single shortcut
- **Testing routines** — Lint, build, and run tests across packages
- **Infrastructure** — Spin up Docker containers, configure networking, seed data

---

## 🏗️ Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run directly without installing
cargo run -- new
cargo run -- list
```

---

## 📁 Project Structure

```
cmdflow/
├── Cargo.toml          # Dependencies and project metadata
├── Cargo.lock          # Locked dependency versions
└── src/
    ├── main.rs         # CLI entry point with clap subcommands
    ├── storage.rs      # Workflow struct, JSON persistence, CRUD
    ├── history.rs      # Shell history reader (cross-platform)
    ├── executor.rs     # Command execution engine
    └── tui.rs          # Interactive TUI (ratatui + crossterm)
```


