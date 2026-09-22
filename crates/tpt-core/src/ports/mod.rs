//! Core ports (traits) and their associated types.

pub mod clock;
pub mod config;
pub mod ipc;
pub mod notifier;
pub mod presence;
pub mod store;

pub use clock::Clock;
pub use config::ConfigSource;
pub use ipc::IpcTransport;
pub use notifier::{Notification, Notifier};
pub use presence::{PresenceEvent, PresenceSource};
pub use store::Store;

#[cfg(test)]
mod tests {
    use super::{Clock, ConfigSource, IpcTransport, Notification, Notifier, PresenceSource, Store};
    use crate::testing::{
        FakeClock, FakePresence, FakeTransport, InMemoryStore, RecordingNotifier,
        StaticConfigSource,
    };

    #[test]
    fn ports_are_object_safe() {
        let clock: Box<dyn Clock> = Box::new(FakeClock::new(0));
        let mut presence: Box<dyn PresenceSource> = Box::new(FakePresence::default());
        let store: Box<dyn Store> = Box::new(InMemoryStore::default());
        let config: Box<dyn ConfigSource> = Box::new(StaticConfigSource::default());

        let recording = RecordingNotifier::default();
        assert_eq!(recording.notifications().len(), 0);
        let mut notifier: Box<dyn Notifier> = Box::new(recording);

        let transport = FakeTransport::default();
        assert!(transport.outbound().is_empty());
        let mut transport: Box<dyn IpcTransport> = Box::new(transport);

        assert!(clock.now_wall() >= 0);
        assert!(presence.poll().is_none());
        assert!(config.load().is_ok());
        assert!(store.list_activities(false).is_ok());
        assert!(
            notifier
                .notify(&Notification::ScreenOff { elapsed_seconds: 0 })
                .is_ok()
        );
        assert!(transport.recv().is_ok());
    }
}
