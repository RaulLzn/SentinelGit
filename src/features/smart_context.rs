use crate::core::GitRepository;
use std::collections::HashMap;

pub fn suggest_prefix(repo: &GitRepository) -> String {
    if let Ok(statuses) = repo.status() {
        if statuses.is_empty() {
            return "chore:".to_string();
        }

        let mut scopes = HashMap::new();
        for (path, _) in statuses {
            let parts: Vec<&str> = path.split('/').collect();
            if let Some(top_level) = parts.first() {
                // If it's src, look deeper
                if *top_level == "src" && parts.len() > 1 {
                    *scopes.entry(parts[1].to_string()).or_insert(0) += 1;
                } else {
                    *scopes.entry(top_level.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Find most common scope
        if let Some((scope, _)) = scopes.iter().max_by_key(|&(_, count)| count) {
            return format!("feat({}):", scope);
        }
    }
    "feat:".to_string()
}
