use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ReplicaId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Lamport {
    pub replica: ReplicaId,
    pub value: u32,
}

impl Lamport {
    pub const LOCAL: Lamport = Lamport {
        replica: 0,
        value: 0,
    };
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionVector(pub HashMap<ReplicaId, u32>);

impl VersionVector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&mut self, ts: Lamport) {
        let entry = self.0.entry(ts.replica).or_insert(0);
        if ts.value > *entry {
            *entry = ts.value;
        }
    }

    pub fn get(&self, replica: ReplicaId) -> u32 {
        *self.0.get(&replica).unwrap_or(&0)
    }

    pub fn includes(&self, ts: Lamport) -> bool {
        self.get(ts.replica) >= ts.value
    }

    /// True iff self includes every entry in `other`.
    pub fn covers(&self, other: &VersionVector) -> bool {
        other.0.iter().all(|(r, v)| self.get(*r) >= *v)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LamportClock {
    pub replica: ReplicaId,
    pub value: u32,
}

impl LamportClock {
    pub fn new(replica: ReplicaId) -> Self {
        Self { replica, value: 0 }
    }

    pub fn tick(&mut self) -> Lamport {
        self.value += 1;
        Lamport {
            replica: self.replica,
            value: self.value,
        }
    }

    pub fn observe(&mut self, ts: Lamport) {
        if ts.value > self.value {
            self.value = ts.value;
        }
    }
}
