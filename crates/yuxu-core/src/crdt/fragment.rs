use crate::crdt::lamport::Lamport;

/// A contiguous slice of an Insertion's text, possibly tombstoned.
#[derive(Debug, Clone)]
pub struct Fragment {
    pub insertion: Lamport,
    pub start: u64,
    pub len: u64,
    /// Tombstones: operations that deleted this fragment. Fragment visible iff empty
    /// *and* its insertion has not been undone.
    pub deletions: Vec<Lamport>,
}

impl Fragment {
    pub fn visible(&self) -> bool {
        self.deletions.is_empty()
    }
}
