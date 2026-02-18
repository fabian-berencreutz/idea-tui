# idea-tui

A high-performance, minimalist terminal-based project manager and launcher for IntelliJ IDEA Ultimate Edition.

Designed to simplify the workflow for developers who manage multiple projects across different categories (Java, Rust, etc.) and frequently clone repositories from GitHub.

## Features

- **📂 Structured Project Browser**: Navigate through your projects by category (e.g., `~/dev/java`, `~/dev/rust`).
- **🚀 One-Click Launch**: Open any directory in IntelliJ directly with a single keystroke.
- **🔗 Smart Git Cloning**: Paste a repository URL, select a category, and let the app clone and open it for you automatically.
- **🔐 Private Repo Support**: Seamlessly integrates with GitHub CLI (`gh`) to handle private repositories without manual password entry.
- **✨ Clean Experience**: All external commands (IntelliJ, Git) run silently in the background, keeping your terminal clutter-free.
- **⌨️ Keyboard Centric**: Full support for arrow keys and Vim-style navigation (`h`, `j`, `k`, `l`).

## Prerequisites

1.  **IntelliJ IDEA Ultimate**: Ensure the launcher is located at `/opt/intellij-idea-ultimate-edition/bin/idea` (standard for many Arch Linux installs).
2.  **GitHub CLI (Optional but Recommended)**: To clone private repositories effortlessly:
    ```bash
    sudo pacman -S github-cli
    gh auth login
    ```

## Installation

1.  Clone this repository.
2.  Build the binary:
    ```bash
    cargo build --release
    ```
3.  Add an alias to your `~/.zshrc` (or `.bashrc`) for easy access:
    ```bash
    alias idea-tui='/home/fabian/dev/rust/intellij-launcher/target/release/intellij-launcher'
    ```

## Navigation & Shortcuts

| Key | Action |
| :--- | :--- |
| **Arrows / hjkl** | Navigate through menus and lists |
| **Enter** | Select action / Enter category / Open project in IntelliJ |
| **Backspace / h** | Go back to the previous menu/category |
| **q** | Quit the application |
| **Esc** | Return to the Main Menu |

## Development Setup

The application starts looking for projects in `/home/fabian/dev`. To make it work for your own structure, you can adjust the base path in `src/main.rs`:

```rust
let mut app = App::new(PathBuf::from("/home/your-user/your-dev-folder"));
```

## How the Cloning Works

When you choose **Clone Repository**, the app will:
1.  Prompt you for a Git URL (HTTPS or SSH).
2.  Ask you to select a destination category (folder in your dev directory).
3.  Attempt to use `gh repo clone` for a seamless experience (especially for private repos).
4.  Fallback to standard `git clone` if `gh` is not available.
5.  Automatically launch the newly cloned project in IntelliJ.
