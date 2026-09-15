//! Command-line client for the Terminal Productivity Tracker.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Returns the version of the `tpt-cli` crate.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn tpt_cli_smoke_lib_exposes_version() {
        assert!(!version().is_empty());
    }
}
