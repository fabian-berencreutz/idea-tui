# idea-tui

A high-performance, minimalist terminal-based project manager and launcher for IntelliJ IDEA Ultimate Edition.

Designed to simplify the workflow for developers who manage multiple projects across different categories (Java, Rust, etc.) and frequently clone repositories from GitHub.

## Features

- **📂 Structured Project Browser**: Navigate through your projects by category (e.g., `~/dev/java`, `~/dev/rust`).
- **🔍 Instant Search**: Press `/` to filter through categories or projects instantly.
- **🚀 One-Click Launch**: Open any directory in IntelliJ directly with a single keystroke.
- **🔗 Smart Git Cloning**: Paste a repository URL, select a category, and let the app clone and open it for you automatically.
- **🔐 Private Repo Support**: Seamlessly integrates with GitHub CLI (`gh`) to handle private repositories without manual password entry.
- **⚙️ Configurable**: Easily customize your project paths and IntelliJ location via a simple config file.
- **⌨️ Keyboard Centric**: Full support for arrow keys and Vim-style navigation (`h`, `j`, `k`, `l`).

## Installation

1.  Clone this repository.
2.  Build the binary:
    ```bash
    cargo build --release
    ```
3.  Add an alias to your `~/.zshrc` (or `.bashrc`) for easy access:
    ```bash
    alias idea-tui='/home/fabian/dev/rust/idea-tui/target/release/idea-tui'
    ```

## Configuration

On the first run, `idea-tui` creates a configuration file at:
`~/.config/idea-tui/default-config.toml`

You can edit this file to match your system:
```toml
base_dir = "/home/your-user/dev"
idea_path = "/usr/bin/idea"
```

## Navigation & Shortcuts

| Key | Action |
| :--- | :--- |
| **Arrows / hjkl** | Navigate through menus and lists |
| **Enter** | Select action / Enter category / Open project |
| **/** | Start searching / filtering (in categories/projects) |
| **Backspace / h** | Go back to the previous menu/category |
| **q** | Quit the application |
| **Esc** | Clear search or return to the Main Menu |

## Prerequisites

1.  **IntelliJ IDEA Ultimate**: The app is pre-configured for the standard Arch Linux path.
2.  **GitHub CLI (Optional)**: To clone private repositories effortlessly:
    ```bash
    sudo pacman -S github-cli
    gh auth login
    ```
