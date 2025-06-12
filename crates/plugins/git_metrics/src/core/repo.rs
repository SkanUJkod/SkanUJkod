use gix::{discover, Repository};

pub struct RepoWrapper {
    pub repo: Repository,
}

impl RepoWrapper {
    pub fn new(path: &str) -> Result<Self, String> {
        let repo = discover(path).map_err(|e| format!("Failed to open repo: {e}"))?;
        Ok(Self { repo })
    }
}
