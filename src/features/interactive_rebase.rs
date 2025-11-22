use crate::core::GitRepository;

#[derive(Debug, Clone)]
pub enum Action {
    Pick,
    Squash,
    Drop,
}

pub struct RebaseEntry {
    pub id: String,
    pub message: String,
    pub action: Action,
}

pub fn load_commits(repo: &GitRepository) -> Vec<RebaseEntry> {
    if let Ok(commits) = repo.get_recent_commits(10) {
        commits
            .into_iter()
            .map(|(id, message)| RebaseEntry {
                id,
                message,
                action: Action::Pick,
            })
            .collect()
    } else {
        vec![]
    }
}
