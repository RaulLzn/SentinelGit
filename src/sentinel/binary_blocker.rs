use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn is_binary(path_str: &str, extensions: &[String]) -> bool {
    let path = Path::new(path_str);

    // 1. Check extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if extensions.iter().any(|e| e == &ext_str) {
            return true;
        }
    }

    // 2. Check content (Null bytes)
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0; 1024];
        if let Ok(n) = file.read(&mut buffer) {
            // Check for null bytes in the first 1024 bytes
            // A common heuristic is that text files don't contain null bytes
            // (except maybe UTF-16, but we assume UTF-8/ASCII for now)
            if buffer[..n].contains(&0) {
                return true;
            }
        }
    }

    false
}
