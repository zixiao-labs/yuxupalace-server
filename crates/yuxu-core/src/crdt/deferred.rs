use crate::crdt::buffer::Operation;
use std::collections::VecDeque;

/// Holds operations whose prerequisite version vector is not yet satisfied.
#[derive(Debug, Default)]
pub struct DeferredOps(pub VecDeque<Operation>);

impl DeferredOps {
    pub fn push(&mut self, op: Operation) {
        self.0.push_back(op);
    }
    pub fn drain(&mut self) -> Vec<Operation> {
        self.0.drain(..).collect()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
