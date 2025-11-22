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
    // 1. Start Chronos Daemon
    std::thread::spawn(|| {
        if let Err(e) = sgit::chronos::watcher::watch(".") {
            eprintln!("Error in Chronos Daemon: {}", e);
        }
    });

    // 2. Start the TUI
    if let Err(e) = ui::dashboard::run() {
        eprintln!("Error running TUI: {}", e);
    }
}
