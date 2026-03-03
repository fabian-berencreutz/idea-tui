# idea-tui 🚀

A high-performance, minimalist terminal-based project manager and launcher for IntelliJ IDEA Ultimate Edition.

## ✨ Features

- **📂 Structured Project Browser**: Navigate through your projects by category (defaults to `~/dev`).
- **🎨 11 Beautiful Themes**: Choose between **Darcula (default)**, Catppuccin, Nord, Tokyo Night, Gruvbox, and more.
- **🛡️ Startup Validation**: Automatically checks if your project and IntelliJ paths are valid on launch.
- **⭐️ Favorites**: Mark your most important projects for instant access.
- **🕒 Recently Opened**: Tracks and lists your last 10 opened projects.
- **🔍 Smart Search**: Press `/` to filter any list instantly.
- **🌿 Real-time Git Status**: See current branch (``), clean status (``), and pending changes (``).
- **🖥️ Quick Terminal**: Press `t` to open your preferred terminal in the project's directory.
- **⚙️ Fully Configurable**: Customize paths, terminal commands, and themes via a TOML config.

## 📸 Screenshots

![A screenshot of idea-tui in action, showing the main menu and project list with git status and language icons.](assets/idea-tui.png)

## 🛠️ Prerequisites

1.  **IntelliJ IDEA Ultimate**: Ensure it's installed. The app will check both `/opt` and your system PATH.
2.  **Nerd Fonts**: Required for icons (``, ``, ``, etc.).
3.  **GitHub CLI (Optional)**: For seamless repo cloning:
    ```bash
    # Example for Arch Linux
    sudo pacman -S github-cli
    gh auth login
    ```

## 🚀 Installation

1.  **Clone this repository**:
    ```bash
    git clone https://github.com/fabian-berencreutz/idea-tui.git
    cd idea-tui
    ```

2.  **Build the binary** (requires [Rust](https://rustup.rs/)):
    ```bash
    cargo build --release
    ```

3.  **Add to PATH** or create an alias in your shell config (`.zshrc` / `.bashrc`):
    ```bash
    # Example for alias
    alias idea-tui='/path/to/idea-tui/target/release/idea-tui'

    # Or move it to a folder in your PATH
    sudo cp target/release/idea-tui /usr/local/bin/
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
| **r** | Refresh Git Status / View |
| **?** | Toggle Help Screen |
| **q** | Quit |
| **Esc** | Clear Search / Main Menu / Close Popups |

## ⚙️ Configuration

On first run, `idea-tui` creates a configuration file at:
`~/.config/idea-tui/default-config.toml`

```toml
base_dir = "/home/user/dev"
idea_path = "/opt/intellij-idea-ultimate-edition/bin/idea"
terminal_command = "kitty --directory" # Command to launch terminal
theme = "Darcula (default)"            # Choose from 11 available themes
```

## 🎨 Themes

Available themes:
`Darcula (default)`, `Catppuccin Mocha`, `Dracula`, `Gruvbox`, `Nord`, `Solarized Dark`, `One Dark`, `Tokyo Night`, `Everforest`, `Rose Pine`, `Ayu Mirage`.

## 🧪 Development

This project is built with **Rust** and follows a modular architecture:
- `app.rs`: State management and business logic.
- `ui.rs`: TUI rendering and layout.
- `models.rs`: Data structures and configuration.
- `theme.rs`: Color definitions.
