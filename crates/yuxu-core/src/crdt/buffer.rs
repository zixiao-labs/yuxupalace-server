use crate::crdt::anchor::{Anchor, Bias};
use crate::crdt::deferred::DeferredOps;
use crate::crdt::fragment::Fragment;
use crate::crdt::lamport::{Lamport, LamportClock, ReplicaId, VersionVector};
use crate::crdt::undo_map::UndoMap;
use crate::{Error, Result};
use std::collections::HashMap;

/// One insertion carries an immutable chunk of text identified by its Lamport id.
#[derive(Debug, Clone)]
pub struct Insertion {
    pub id: Lamport,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Edit {
        timestamp: Lamport,
        version: VersionVector,
        ranges: Vec<(Anchor, Anchor)>,
        new_text: Vec<String>,
    },
    Undo {
        timestamp: Lamport,
        version: VersionVector,
        counts: Vec<(Lamport, u32)>,
    },
}

impl Operation {
    pub fn timestamp(&self) -> Lamport {
        match self {
            Operation::Edit { timestamp, .. } | Operation::Undo { timestamp, .. } => *timestamp,
        }
    }
    pub fn version(&self) -> &VersionVector {
        match self {
            Operation::Edit { version, .. } | Operation::Undo { version, .. } => version,
        }
    }
}

pub struct Buffer {
    pub clock: LamportClock,
    pub version: VersionVector,
    pub insertions: HashMap<Lamport, Insertion>,
    pub fragments: Vec<Fragment>,
    pub undo_map: UndoMap,
    pub deferred: DeferredOps,
}

impl Buffer {
    pub fn new(replica: ReplicaId, initial_text: &str) -> Self {
        let mut buf = Self {
            clock: LamportClock::new(replica),
            version: VersionVector::new(),
            insertions: HashMap::new(),
            fragments: Vec::new(),
            undo_map: UndoMap::default(),
            deferred: DeferredOps::default(),
        };
        if !initial_text.is_empty() {
            let ts = buf.clock.tick();
            buf.version.observe(ts);
            buf.insertions.insert(
                ts,
                Insertion {
                    id: ts,
                    text: initial_text.to_string(),
                },
            );
            buf.fragments.push(Fragment {
                insertion: ts,
                start: 0,
                len: initial_text.len() as u64,
                deletions: Vec::new(),
            });
        }
        buf
    }

    pub fn replica(&self) -> ReplicaId {
        self.clock.replica
    }

    /// Current visible text. O(n) over fragments.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for f in &self.fragments {
            if !self.is_fragment_visible(f) {
                continue;
            }
            if let Some(ins) = self.insertions.get(&f.insertion) {
                out.push_str(&ins.text[f.start as usize..(f.start + f.len) as usize]);
            }
        }
        out
    }

    fn is_fragment_visible(&self, f: &Fragment) -> bool {
        if self.undo_map.is_undone(f.insertion) {
            return false;
        }
        for d in &f.deletions {
            if !self.undo_map.is_undone(*d) {
                return false;
            }
        }
        true
    }

    /// Offset in visible text → (fragment_index, offset_within_fragment).
    fn locate_visible_offset(&self, mut target: u64) -> (usize, u64) {
        for (i, f) in self.fragments.iter().enumerate() {
            if !self.is_fragment_visible(f) {
                continue;
            }
            if target <= f.len {
                return (i, target);
            }
            target -= f.len;
        }
        (self.fragments.len(), 0)
    }

    /// Build an Anchor at a given visible-text offset.
    pub fn anchor_at(&self, visible_offset: u64, bias: Bias) -> Anchor {
        let (idx, within) = self.locate_visible_offset(visible_offset);
        if idx >= self.fragments.len() {
            // End of document.
            if let Some(f) = self.fragments.last() {
                return Anchor {
                    insertion: f.insertion,
                    offset: f.start + f.len,
                    bias,
                };
            }
            return Anchor::START;
        }
        let f = &self.fragments[idx];
        Anchor {
            insertion: f.insertion,
            offset: f.start + within,
            bias,
        }
    }

    /// Locate (fragment_index, within_fragment_offset) for an Anchor.
    fn resolve_anchor(&self, a: Anchor) -> (usize, u64) {
        for (i, f) in self.fragments.iter().enumerate() {
            if f.insertion == a.insertion && a.offset >= f.start && a.offset <= f.start + f.len {
                return (i, a.offset - f.start);
            }
        }
        (self.fragments.len(), 0)
    }

    /// Split fragment at local index `idx` at offset `within` (0..=len).
    /// Returns (new_index_of_right_half, inserted_new_fragment).
    fn split_at(&mut self, idx: usize, within: u64) -> (usize, bool) {
        if idx >= self.fragments.len() {
            return (idx, false);
        }
        let f = self.fragments[idx].clone();
        if within == 0 {
            return (idx, false);
        }
        if within == f.len {
            return (idx + 1, false);
        }
        let left = Fragment {
            insertion: f.insertion,
            start: f.start,
            len: within,
            deletions: f.deletions.clone(),
        };
        let right = Fragment {
            insertion: f.insertion,
            start: f.start + within,
            len: f.len - within,
            deletions: f.deletions,
        };
        self.fragments[idx] = left;
        self.fragments.insert(idx + 1, right);
        (idx + 1, true)
    }

    /// Apply a local edit expressed over visible-text offsets. Returns the produced Operation
    /// (to be broadcast to peers).
    pub fn local_edit(
        &mut self,
        ranges: Vec<(u64, u64)>,
        new_text: Vec<String>,
    ) -> Result<Operation> {
        if ranges.len() != new_text.len() {
            return Err(Error::BadRequest(
                "ranges / new_text length mismatch".into(),
            ));
        }
        let version = self.version.clone();
        let ts = self.clock.tick();
        self.version.observe(ts);
        let anchor_ranges: Vec<(Anchor, Anchor)> = ranges
            .iter()
            .map(|(s, e)| {
                (
                    self.anchor_at(*s, Bias::Left),
                    self.anchor_at(*e, Bias::Right),
                )
            })
            .collect();

        self.apply_edit_internal(ts, &anchor_ranges, &new_text);

        Ok(Operation::Edit {
            timestamp: ts,
            version,
            ranges: anchor_ranges,
            new_text,
        })
    }

    /// Apply a remote operation. Deferred until prerequisites are satisfied.
    pub fn apply_remote(&mut self, op: Operation) {
        if !self.version.covers(op.version()) {
            self.deferred.push(op);
            return;
        }
        self.apply_one(op);
        self.flush_deferred();
    }

    fn flush_deferred(&mut self) {
        loop {
            let mut progress = false;
            let pending = self.deferred.drain();
            for op in pending {
                if self.version.covers(op.version()) {
                    self.apply_one(op);
                    progress = true;
                } else {
                    self.deferred.push(op);
                }
            }
            if !progress {
                break;
            }
        }
    }

    fn apply_one(&mut self, op: Operation) {
        self.clock.observe(op.timestamp());
        match op {
            Operation::Edit {
                timestamp,
                ranges,
                new_text,
                ..
            } => {
                self.apply_edit_internal(timestamp, &ranges, &new_text);
                self.version.observe(timestamp);
            }
            Operation::Undo {
                timestamp, counts, ..
            } => {
                for (target, count) in counts {
                    self.undo_map.set(target, count);
                }
                self.version.observe(timestamp);
            }
        }
    }

    fn apply_edit_internal(
        &mut self,
        timestamp: Lamport,
        ranges: &[(Anchor, Anchor)],
        new_text: &[String],
    ) {
        // Resolve all ranges to indices *before* any mutation.
        let mut resolved: Vec<(usize, u64, usize, u64)> = ranges
            .iter()
            .map(|(s, e)| {
                let (si, so) = self.resolve_anchor(*s);
                let (ei, eo) = self.resolve_anchor(*e);
                (si, so, ei, eo)
            })
            .collect();

        // Sort indices (descending) so later mutations don't invalidate earlier ones.
        let mut order: Vec<usize> = (0..resolved.len()).collect();
        order.sort_by(|a, b| {
            (resolved[*b].0, resolved[*b].1).cmp(&(resolved[*a].0, resolved[*a].1))
        });

        for idx in order {
            let (si, so, ei, eo) = resolved[idx];
            let text = &new_text[idx];

            // Split at the end first (indices before `ei` remain valid).
            let (right_cut, _) = if ei < self.fragments.len() {
                self.split_at(ei, eo)
            } else {
                (self.fragments.len(), false)
            };
            // Then split at the start.
            let (left_cut, start_inserted) = if si < self.fragments.len() {
                self.split_at(si, so)
            } else {
                (self.fragments.len(), false)
            };
            // If the start split inserted a new fragment before the right cut, bump it.
            let right_cut = if start_inserted && si <= ei {
                right_cut + 1
            } else {
                right_cut
            };

            // Tombstone fragments in [left_cut, right_cut).
            let end = right_cut.min(self.fragments.len());
            for i in left_cut..end {
                self.fragments[i].deletions.push(timestamp);
            }

            // Insert new text as a fresh insertion at left_cut.
            if !text.is_empty() {
                self.insertions.insert(
                    timestamp,
                    Insertion {
                        id: timestamp,
                        text: text.clone(),
                    },
                );
                let frag = Fragment {
                    insertion: timestamp,
                    start: 0,
                    len: text.len() as u64,
                    deletions: Vec::new(),
                };
                self.fragments.insert(left_cut, frag);
            }
            let _ = &mut resolved;
        }
    }

    /// Issue an undo/redo operation toggling the given target operations.
    pub fn local_undo(&mut self, targets: Vec<Lamport>) -> Operation {
        let version = self.version.clone();
        let ts = self.clock.tick();
        self.version.observe(ts);
        let mut counts = Vec::new();
        for t in targets {
            let next = self.undo_map.0.get(&t).copied().unwrap_or(0) + 1;
            self.undo_map.set(t, next);
            counts.push((t, next));
        }
        Operation::Undo {
            timestamp: ts,
            version,
            counts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_replica_insert() {
        let mut a = Buffer::new(1, "hello");
        let _ = a.local_edit(vec![(5, 5)], vec![" world".into()]).unwrap();
        assert_eq!(a.text(), "hello world");
    }

    #[test]
    fn delete_then_text() {
        let mut a = Buffer::new(1, "hello world");
        let _ = a.local_edit(vec![(5, 11)], vec!["".into()]).unwrap();
        assert_eq!(a.text(), "hello");
    }

    #[test]
    fn two_replicas_converge() {
        let _a = Buffer::new(1, "ab");
        let _b = Buffer::new(2, "ab");
        // Bring b in sync with a's initial insertion by cross-applying.
        // Simpler path: start both from empty and exchange.
        let mut a = Buffer::new(1, "");
        let mut b = Buffer::new(2, "");
        let op1 = a.local_edit(vec![(0, 0)], vec!["hello".into()]).unwrap();
        b.apply_remote(op1.clone());
        let op2 = b.local_edit(vec![(5, 5)], vec![" world".into()]).unwrap();
        a.apply_remote(op2);
        assert_eq!(a.text(), b.text());
        assert_eq!(a.text(), "hello world");
    }

    #[test]
    fn deferred_op_is_applied_when_prereq_arrives() {
        let mut a = Buffer::new(1, "");
        let mut b = Buffer::new(2, "");
        let op1 = a.local_edit(vec![(0, 0)], vec!["ab".into()]).unwrap();
        let op2 = a.local_edit(vec![(2, 2)], vec!["cd".into()]).unwrap();
        // Deliver out of order.
        b.apply_remote(op2);
        assert_eq!(b.text(), ""); // still deferred
        b.apply_remote(op1);
        assert_eq!(b.text(), "abcd");
    }

    #[test]
    fn undo_hides_insertion() {
        let mut a = Buffer::new(1, "");
        let op = a.local_edit(vec![(0, 0)], vec!["X".into()]).unwrap();
        let ts = op.timestamp();
        assert_eq!(a.text(), "X");
        let _ = a.local_undo(vec![ts]);
        assert_eq!(a.text(), "");
    }
}
