//! IPC transport port. The protocol itself is Phase 6.

use crate::error::IpcError;

/// Byte-level transport between clients and the daemon.
pub trait IpcTransport {
    fn send(&mut self, payload: &[u8]) -> Result<(), IpcError>;
    fn recv(&mut self) -> Result<Option<Vec<u8>>, IpcError>;
}
