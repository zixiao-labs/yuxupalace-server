use crate::crdt::lamport::Lamport;
use std::collections::HashMap;

/// operation_id → count. Odd = undone, even > 0 = redone, 0 = never touched.
#[derive(Debug, Clone, Default)]
pub struct UndoMap(pub HashMap<Lamport, u32>);

impl UndoMap {
    pub fn is_undone(&self, op: Lamport) -> bool {
        self.0.get(&op).copied().unwrap_or(0) % 2 == 1
    }
    pub fn set(&mut self, op: Lamport, count: u32) {
        self.0.insert(op, count);
    }
}
