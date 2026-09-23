//! Adapters that connect the core ports to the operating system and to storage.
//!
//! Ports live in `tpt-core`; every concrete implementation lives here (ADR-003).

pub mod sqlite;
