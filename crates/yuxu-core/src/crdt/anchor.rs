use crate::crdt::lamport::Lamport;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bias {
    Left,
    Right,
}

/// An anchor points at `(insertion_id, offset_within_insertion, bias)`.
/// Anchors are stable even as the document is edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    pub insertion: Lamport,
    pub offset: u64,
    pub bias: Bias,
}

impl Anchor {
    pub const START: Anchor = Anchor {
        insertion: Lamport {
            replica: 0,
            value: 0,
        },
        offset: 0,
        bias: Bias::Left,
    };
    pub const END: Anchor = Anchor {
        insertion: Lamport {
            replica: 0,
            value: u32::MAX,
        },
        offset: u64::MAX,
        bias: Bias::Right,
    };
}
