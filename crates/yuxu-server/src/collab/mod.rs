//! Collaboration hub: connection registry + room/project state + Envelope routing.

pub mod envelope;
pub mod hub;
pub mod livekit;
pub mod project;
pub mod room;
pub mod ws;

pub use hub::CollabHub;
