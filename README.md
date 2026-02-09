# SentinelGit 🛡️

SentinelGit is a high-performance, terminal-based Git client designed to elevate the development workflow through proactive security, granular history management, and intelligent context awareness. Unlike traditional Git clients that act merely as wrappers around Git commands, SentinelGit actively guards your codebase and enhances your decision-making process.

![SentinelGit Screenshot](https://via.placeholder.com/800x400?text=SentinelGit+Dashboard)

## 🚀 Key Features

### 🛡️ The Guardian (Sentinel)

SentinelGit integrates a real-time security engine that scans files before they enter your staging area.

- **Secret Detection**: Automatically identifies and blocks potential secrets (AWS keys, private keys, etc.).
- **Binary Blocker**: Prevents accidental staging of binary files.
- **Proactive Defense**: Staging is blocked at the source if a threat is detected.

### ⏳ The Time Machine (Chronos) & Ghost Branches

Chronos provides a safety net beyond standard Git commits.

- **Background Watcher**: A lightweight daemon monitors your workspace for changes in real-time.
- **Ghost Branches**: Every modification is automatically compressed and saved.
- **Time Travel**: Press `t` to open the Time Machine modal and restore your entire project to any previous state, even if you never committed it.
- **File History**: Press `h` to see the revision history of a specific file and restore it individually.

### 📝 Smart Commit Wizard

SentinelGit enforces and simplifies Conventional Commits.

- **Interactive Wizard**: Press `c` to open a multi-step wizard that guides you through type, scope, summary, and description.
- **Automated Formatting**: Generates standardized commit messages (e.g., `feat(ui): add new button`).

### 🔍 Interactive Staging (Diff Viewer)

Review and stage changes with precision.

- **Hunk Selection**: Press `d` to view a file's diff. Use `Up`/`Down` to navigate individual changes (hunks).
- **Partial Staging**: Press `s` to stage only the selected hunk, allowing for atomic commits.
- **Syntax Highlighting**: clear, color-coded diffs.

### 🧘 Zen Mode & Impact Radar

- **Zen Mode**: Press `z` to remove all distractions and focus on your code.
- **Impact Score**: Calculates the cognitive load and risk of your changes.

## ⌨️ Usage

| Key       | Action                                              |
| :-------- | :-------------------------------------------------- |
| `↑` / `↓` | Navigate file list                                  |
| `Space`   | Stage/Unstage file (triggers Sentinel checks)       |
| `d`       | **Diff View**: Interactive staging (Hunk selection) |
| `c`       | **Commit**: Open Conventional Commits Wizard        |
| `h`       | **History**: View/Restore file snapshots            |
| `t`       | **Time Machine**: Restore project to previous state |
| `z`       | Toggle Zen Mode                                     |
| `?`       | Show Help / Keyboard Shortcuts                      |
| `q`       | Quit                                                |
| `Esc`     | Close Modal / Cancel                                |

## 📦 Installation

Ensure you have **Rust** and **Cargo** installed.

### 1. Automated Install (Recommended)

Run the installation script to build and install `sg` to your local bin directory:

```bash
./install.sh
```

Restart your terminal, then run:

```bash
sg
```

### 2. Manual Build

```bash
cargo build --release
./target/release/sentinel-git
```

## 🔧 Configuration

SentinelGit uses a `.sgit.toml` file for configuration. You can customize:

- **Sentinel Rules**: Define custom regex patterns for secrets.
- **Ignored Files**: Manage binary extensions to block.
- **Chronos Settings**: Adjust snapshot frequency and retention.

## 🤝 Contributing

Contributions are welcome! Please read our [CONTRIBUTING.md](CONTRIBUTING.md) (coming soon) for details on our code of conduct, and the process for submitting pull requests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
