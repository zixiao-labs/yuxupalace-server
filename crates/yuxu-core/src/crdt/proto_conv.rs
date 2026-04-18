//! Conversions between in-memory CRDT types and raidian collab protobuf types.

use crate::crdt::anchor::{Anchor, Bias};
use crate::crdt::buffer::Operation;
use crate::crdt::lamport::{Lamport, VersionVector};
use raidian::collab as pb;

impl From<Lamport> for pb::LamportTimestamp {
    fn from(l: Lamport) -> Self {
        pb::LamportTimestamp {
            replica_id: l.replica,
            value: l.value,
        }
    }
}
impl From<pb::LamportTimestamp> for Lamport {
    fn from(p: pb::LamportTimestamp) -> Self {
        Lamport {
            replica: p.replica_id,
            value: p.value,
        }
    }
}

pub fn version_to_pb(v: &VersionVector) -> Vec<pb::VectorClockEntry> {
    v.0.iter()
        .map(|(r, t)| pb::VectorClockEntry {
            replica_id: *r,
            timestamp: *t,
        })
        .collect()
}
pub fn version_from_pb(entries: &[pb::VectorClockEntry]) -> VersionVector {
    let mut vv = VersionVector::new();
    for e in entries {
        vv.0.insert(e.replica_id, e.timestamp);
    }
    vv
}

impl From<Bias> for i32 {
    fn from(b: Bias) -> i32 {
        match b {
            Bias::Left => pb::Bias::Left as i32,
            Bias::Right => pb::Bias::Right as i32,
        }
    }
}
pub fn bias_from_pb(v: i32) -> Bias {
    if v == pb::Bias::Right as i32 {
        Bias::Right
    } else {
        Bias::Left
    }
}

impl From<Anchor> for pb::Anchor {
    fn from(a: Anchor) -> Self {
        pb::Anchor {
            timestamp: Some(a.insertion.into()),
            offset: a.offset,
            bias: a.bias.into(),
            buffer_id: None,
        }
    }
}
pub fn anchor_from_pb(a: &pb::Anchor) -> Anchor {
    Anchor {
        insertion: a.timestamp.map(Into::into).unwrap_or(Lamport::LOCAL),
        offset: a.offset,
        bias: bias_from_pb(a.bias),
    }
}

pub fn op_to_pb(op: &Operation) -> pb::Operation {
    use pb::operation::{Edit, Undo, Variant};
    match op {
        Operation::Edit {
            timestamp,
            version,
            ranges,
            new_text,
        } => pb::Operation {
            variant: Some(Variant::Edit(Edit {
                timestamp: Some((*timestamp).into()),
                version: version_to_pb(version),
                ranges: ranges
                    .iter()
                    .map(|(s, e)| pb::Range {
                        start: Some((*s).into()),
                        end: Some((*e).into()),
                    })
                    .collect(),
                new_text: new_text.clone(),
            })),
        },
        Operation::Undo {
            timestamp,
            version,
            counts,
        } => pb::Operation {
            variant: Some(Variant::Undo(Undo {
                timestamp: Some((*timestamp).into()),
                version: version_to_pb(version),
                counts: counts
                    .iter()
                    .map(|(t, c)| pb::UndoCount {
                        operation_timestamp: Some((*t).into()),
                        count: *c,
                    })
                    .collect(),
            })),
        },
    }
}

pub fn op_from_pb(op: &pb::Operation) -> Option<Operation> {
    use pb::operation::Variant;
    match op.variant.as_ref()? {
        Variant::Edit(e) => Some(Operation::Edit {
            timestamp: e.timestamp?.into(),
            version: version_from_pb(&e.version),
            ranges: e
                .ranges
                .iter()
                .filter_map(|r| {
                    let s = r.start.as_ref()?;
                    let en = r.end.as_ref()?;
                    Some((anchor_from_pb(s), anchor_from_pb(en)))
                })
                .collect(),
            new_text: e.new_text.clone(),
        }),
        Variant::Undo(u) => Some(Operation::Undo {
            timestamp: u.timestamp?.into(),
            version: version_from_pb(&u.version),
            counts: u
                .counts
                .iter()
                .filter_map(|c| Some((c.operation_timestamp?.into(), c.count)))
                .collect(),
        }),
        _ => None,
    }
}
