use crate::core::GitRepository;

pub struct ImpactScore {
    pub insertions: usize,
    pub deletions: usize,
    pub score: f64,
    pub level: String,
}

pub fn scan_changes(repo: &GitRepository) -> Option<ImpactScore> {
    if let Ok((insertions, deletions)) = repo.get_diff_stats() {
        let total = insertions + deletions;
        let score = (total as f64).log10() * 10.0; // Simple logarithmic score

        let level = if score > 50.0 {
            "CRITICAL".to_string()
        } else if score > 30.0 {
            "HIGH".to_string()
        } else if score > 10.0 {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        };

        Some(ImpactScore {
            insertions,
            deletions,
            score,
            level,
        })
    } else {
        None
    }
}
