use sgit::ui;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: Option<String>,
}

fn main() {
    let args = Args::parse();
    println!("SentinelGit (sgit) v0.1.0");
    // Placeholder for TUI start
    // ui::start();
}
