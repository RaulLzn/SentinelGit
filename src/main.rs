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

    // 1. Start Chronos Daemon
    let config_clone = config.clone();
    std::thread::spawn(move || {
        if let Err(e) = sgit::chronos::watcher::watch(".", &config_clone) {
            eprintln!("Error in Chronos Daemon: {}", e);
        }
    });

    // 2. Start the TUI
    if let Err(e) = ui::dashboard::run() {
        eprintln!("Error running TUI: {}", e);
    }
}
