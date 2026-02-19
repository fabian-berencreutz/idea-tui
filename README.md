# idea-tui 🚀

A high-performance, minimalist terminal-based project manager and launcher for IntelliJ IDEA Ultimate Edition, featuring a beautiful Catppuccin Mocha theme.

## ✨ Features

- **📂 Structured Project Browser**: Navigate through your projects by category (e.g., `~/dev/java`).
- **⭐️ Favorites**: Mark your most important projects with `f` for instant access.
- **🕒 Recently Opened**: Automatically tracks and lists your last 10 projects.
- **🔍 Smart Search**: Press `/` to filter any list. Confirm with `Enter` to browse the results.
- **🌿 Real-time Git Status**: See your current branch (``), checkmarks (``) for clean repos, and dots (``) for pending changes.
- **🖥️ Quick Terminal**: Press `t` to instantly open a new terminal window in the project's directory.
- **🛡️ Confirmation Safety**: A built-in popup ensures you only launch IntelliJ when you actually mean to.
- **🎨 Catppuccin Mocha**: Beautifully themed with Teal borders, Blue highlights, and Peach accents.
- **⚙️ Fully Configurable**: Customize your paths and terminal commands via a simple TOML file.

## 🛠️ Prerequisites

1.  **IntelliJ IDEA Ultimate**: Ensure it's installed (standard path: `/opt/intellij-idea-ultimate-edition/bin/idea`).
2.  **Nerd Fonts**: Required for icons (``, ``, ``, etc.).
3.  **GitHub CLI (Optional)**: For seamless private repo cloning:
    ```bash
    sudo pacman -S github-cli
    gh auth login
    ```

## 🚀 Installation

1.  Clone this repository.
2.  Build the binary:
    ```bash
    cargo build --release
    ```
3.  Add an alias to your `~/.zshrc` (or `.bashrc`):
    ```bash
    alias idea-tui='/home/fabian/dev/rust/idea-tui/target/release/idea-tui'
    ```

## ⌨️ Shortcuts

| Key | Action |
| :--- | :--- |
| **Arrows / hjkl** | Navigate menus and lists |
| **Enter / l** | Select / Enter / Trigger Open |
| **Backspace / h** | Go back / Cancel |
| **/** | Start search (Press **Enter** to browse results) |
| **f** | Toggle Favorite |
| **t** | Open Quick Terminal |
| **?** | Toggle Help Screen |
| **q** | Quit |
| **Esc** | Clear Search / Main Menu / Close Popups |

## ⚙️ Configuration

On first run, `idea-tui` creates a config file at:
`~/.config/idea-tui/default-config.toml`

```toml
base_dir = "/home/fabian/dev"
idea_path = "/opt/intellij-idea-ultimate-edition/bin/idea"
terminal_command = "kitty --directory" # Command to launch terminal + path
```

## 🧪 Development

This project is built with **Rust** and **Ratatui**.

```bash
cargo run   # Run in debug mode
cargo build # Build binary
```
