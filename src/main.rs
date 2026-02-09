use clap::Parser;
use sgit::ui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: Option<String>,
}

fn main() {
    let _args = Args::parse();
    println!("SentinelGit (sgit) v0.1.0");

    let config = sgit::config::Config::load().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}, using defaults", e);
        sgit::config::Config::default()
    });

    // Initialize Chronos Store
    // Using a fixed path in .git/chronos_db for now, or from config if implemented
    let store_path = std::path::Path::new(".git/chronos_db");
    // Ensure parent dir exists or let sled handle it? Sled creates dir.
    // But .git might not exist if we run outside a repo.
    // For now assume we run in a repo root.

    let store = match sgit::chronos::storage::ChronosStore::open(store_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Failed to open Chronos Store: {}. Chronos features disabled.",
                e
            );
            // We need a fallback or abort.
            // Since we changed App to require Store, we must provide one.
            // A better way would be Option<ChronosStore> in App, but let's just panic or exit for now if DB fails
            // as this is a core component now.
            // Or better: create a temporary in-memory one? Sled supports temporary?
            // Let's just exit for simplicity in this prototype phase.
            std::process::exit(1);
        }
    };

    // 1. Start Chronos Daemon
    let config_clone = config.clone();
    let store_clone = store.clone();

    std::thread::spawn(move || {
        if let Err(e) = sgit::chronos::watcher::watch(".", &config_clone, store_clone) {
            eprintln!("Error in Chronos Daemon: {}", e);
        }
    });

    // 2. Start the TUI
    if let Err(e) = ui::dashboard::run(config, store) {
        eprintln!("Error running TUI: {}", e);
    }
}
