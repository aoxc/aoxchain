// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub yes_votes: u64,
    pub no_votes: u64,
}

impl Proposal {
    pub fn accepted(&self) -> bool {
        self.yes_votes > self.no_votes
    }
}
