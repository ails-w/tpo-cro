//! Core domain for the Terminal Productivity Tracker: models, ports (traits) and rules.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Returns the version of the `tpt-core` crate.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn tpt_core_smoke_lib_exposes_version() {
        assert!(!version().is_empty());
    }
}
