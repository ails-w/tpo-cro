//! Clock port.

/// Monotonic + wall-clock time. Detects suspend gaps by comparing both.
pub trait Clock {
    /// Seconds from an arbitrary monotonic origin (never goes backwards, frozen while suspended).
    fn now_monotonic(&self) -> i64;
    /// Seconds since the Unix epoch, UTC.
    fn now_wall(&self) -> i64;
}
