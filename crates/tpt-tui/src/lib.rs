//! Terminal user interface for the Terminal Productivity Tracker.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Returns the version of the `tpt-tui` crate.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn tpt_tui_smoke_lib_exposes_version() {
        assert!(!version().is_empty());
    }
}
