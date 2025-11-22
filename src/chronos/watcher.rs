use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, Event};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                // Filter out .git directory and other noise
                if let Some(path) = event.paths.get(0) {
                    if path.to_string_lossy().contains(".git") || path.to_string_lossy().contains("target") {
                        continue;
                    }
                }
                println!("Change: {:?}", event);
                // Here we would trigger the snapshot logic
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
