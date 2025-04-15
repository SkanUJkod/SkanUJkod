use gix::{ discover, Repository };

pub struct RepoWrapper {
    pub repo: Repository,
}

impl RepoWrapper {
    pub fn new(path: &str) -> Self {
        let repo = discover(path).expect("Failed to open repo");
        Self { repo }
    }
}
