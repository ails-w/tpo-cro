//! Presence port.

/// A change in the user's presence reported by a presence source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceEvent {
    Idle { seconds: i64 },
    Active,
    ScreenOff,
    Suspended { seconds: i64 },
    Resumed,
}

/// Answers one question: was the user present? (ADR-003)
pub trait PresenceSource {
    fn poll(&mut self) -> Option<PresenceEvent>;
}
