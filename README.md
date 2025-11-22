# SentinelGit

SentinelGit is a high-performance, terminal-based Git client designed to elevate the development workflow through proactive security, granular history management, and intelligent context awareness. Unlike traditional Git clients that act merely as wrappers around Git commands, SentinelGit actively guards your codebase and enhances your decision-making process.

## Philosophy

In modern software development, the cost of errors is high. Leaked secrets, accidental binary commits, and vague commit messages are common pitfalls that disrupt workflows and compromise security. SentinelGit addresses these issues by shifting validation left—catching problems before they are staged, not after they are pushed. It is built on the belief that your tools should be active partners in your development process, not just passive utilities.

## Key Features

### The Guardian (Sentinel)

SentinelGit integrates a real-time security engine that scans files before they enter your staging area.

- **Secret Detection**: Automatically identifies and blocks potential secrets, such as AWS keys, private keys, and high-entropy strings, preventing accidental leaks.
- **Binary Blocker**: Prevents the accidental staging of binary files, keeping your repository clean and performant.
- **Proactive Defense**: Staging is blocked at the source if a threat is detected, enforcing security best practices by default.

### The Time Machine (Chronos)

Chronos provides a safety net beyond standard Git commits.

- **Background Watcher**: A lightweight daemon monitors your workspace for changes in real-time.
- **Granular Snapshots**: Every modification is automatically compressed and saved to a local database. This allows you to recover work even if it was never committed or staged, effectively giving you an undo button for your entire project history.

### Impact Radar

Understanding the scope of your changes is crucial for code review and stability.

- **Cognitive Load Analysis**: The Impact Radar analyzes your changes to calculate an impact score, giving you immediate feedback on the complexity and risk associated with your current work.
- **Risk Assessment**: Helps you decide whether a change is too large and should be broken down, promoting better code hygiene.

### Smart Context

SentinelGit reduces the friction of adhering to Conventional Commits.

- **Intelligent Scoping**: The system analyzes the modified files to suggest appropriate commit scopes (e.g., `feat(ui)`, `fix(core)`).
- **Automated Prefixes**: When opening the commit modal, the message field is pre-filled with the suggested context, streamlining the commit process and ensuring consistency in your project history.

### Zen Mode

Designed for flow, Zen Mode removes all distractions from the interface, leaving only the essential file list and status indicators. This allows you to focus entirely on the task at hand without visual clutter.

## Usage

SentinelGit is designed to be intuitive for users familiar with terminal interfaces.

- **Navigation**: Use Up/Down arrows to traverse your file list.
- **Stage/Unstage**: Press Space to toggle the staged status of a file. Sentinel will perform security checks immediately.
- **Commit**: Press 'c' to open the commit modal. Enter your message and press Enter to confirm, or Esc to cancel.
- **Zen Mode**: Press 'z' to toggle the distraction-free view.
- **Quit**: Press 'q' to exit the application.

## Installation

Ensure you have Rust and Cargo installed on your system.

```bash
cargo build --release
./target/release/sgit
```
