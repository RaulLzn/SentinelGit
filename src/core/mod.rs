use anyhow::Result;
use git2::{Repository, StatusOptions};
use std::path::Path;

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::init(path)?;
        Ok(Self { repo })
    }

    pub fn status(&self) -> Result<Vec<(String, String)>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = self.repo.statuses(Some(&mut opts))?;

        let mut result = Vec::new();
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = format!("{:?}", entry.status());
            result.push((path, status));
        }
        Ok(result)
    }

    pub fn add(&self, pathspec: &[&str]) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(pathspec.iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<git2::Oid> {
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let signature = self.repo.signature()?;
        let parent_commit = match self.repo.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => None, // Initial commit
        };

        let parents = match &parent_commit {
            Some(c) => vec![c],
            None => vec![],
        };

        let oid = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;

        Ok(oid)
    }
    pub fn get_stashes(&mut self) -> Result<Vec<String>> {
        let mut stashes = Vec::new();
        self.repo.stash_foreach(|index, name, _oid| {
            stashes.push(format!("stash@{{{}}}: {}", index, name));
            true
        })?;
        Ok(stashes)
    }

    pub fn get_diff_stats(&self) -> Result<(usize, usize)> {
        let mut opts = git2::DiffOptions::new();
        opts.include_untracked(true);

        // Diff index to workdir
        let diff = self.repo.diff_index_to_workdir(None, Some(&mut opts))?;
        let stats = diff.stats()?;

        Ok((stats.insertions(), stats.deletions()))
    }

    pub fn get_recent_commits(&self, limit: usize) -> Result<Vec<(String, String)>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();
        for oid in revwalk.take(limit) {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            let message = commit.summary().unwrap_or("").to_string();
            let short_id = oid.to_string()[..7].to_string();
            commits.push((short_id, message));
        }
        Ok(commits)
    }
}
