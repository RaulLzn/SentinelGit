use crate::core::GitRepository;

pub struct ShelfState {
    pub stashes: Vec<String>,
}

impl ShelfState {
    pub fn new() -> Self {
        Self { stashes: vec![] }
    }

    pub fn refresh(&mut self, repo: &mut GitRepository) {
        if let Ok(stashes) = repo.get_stashes() {
            self.stashes = stashes;
        }
    }
}
