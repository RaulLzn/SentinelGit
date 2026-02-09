use crate::chronos::storage::ChronosStore;
use crate::config::Config; // Import Config
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher}; // Rename notify::Config
use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;

pub fn watch<P: AsRef<Path>>(path: P, config: &Config, store: ChronosStore) -> notify::Result<()> {
    if !config.chronos.enabled {
        println!("Chronos watcher is disabled in config.");
        return Ok(());
    }
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                // Filter out .git directory and other noise
                if let Some(path) = event.paths.get(0) {
                    if path.to_string_lossy().contains(".git")
                        || path.to_string_lossy().contains("target")
                    {
                        continue;
                    }

                    // Only act on Modify or Create events
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            println!("Change detected in: {:?}", path);
                            if let Ok(content) = fs::read(path) {
                                if let Err(e) =
                                    store.save_snapshot(&path.to_string_lossy(), &content)
                                {
                                    println!("Failed to save snapshot: {}", e);
                                } else {
                                    println!("Snapshot saved for {:?}", path);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
