use super::Subscription;
use crate::{session::SessionId, state::Key};
use alloc::sync::{Arc, Weak};
use dashmap::{mapref::entry::Entry, DashMap, DashSet};

#[derive(Debug, Default)]
struct PubSubManagerRef {
    keys: DashMap<Key, DashSet<SessionId>>,
    sessions: DashMap<SessionId, DashMap<Key, Arc<Subscription>>>,
}

#[derive(Clone, Debug, Default)]
pub struct PubSubManager(Arc<PubSubManagerRef>);

impl PubSubManager {
    /// Retrieves a session's subscription for a key, if it exists.
    pub fn get(&self, object_key: &[u8], session_id: SessionId) -> Option<Weak<Subscription>> {
        let session = self.0.sessions.get(&session_id)?;
        let sub = session.get(object_key)?;

        Some(Arc::downgrade(sub.value()))
    }

    /// Subscribes a session by ID to an object key.
    ///
    /// Returns the new subscription if subscribing was successful. Returns None
    /// if the subscription already existed.
    pub fn subscribe(&self, object_key: Key, session_id: SessionId) -> Option<Weak<Subscription>> {
        let session = self.0.sessions.entry(session_id).or_default();

        let subscription = match session.entry(object_key.clone()) {
            Entry::Occupied(_) => return None,
            Entry::Vacant(v) => {
                let subscription = Arc::new(Subscription::new());
                let weak = Arc::downgrade(&subscription);

                v.insert(subscription);

                weak
            }
        };

        match self.0.keys.entry(object_key) {
            Entry::Occupied(_) => None,
            Entry::Vacant(v) => {
                let set = v.insert(DashSet::new());
                set.insert(session_id);

                Some(subscription)
            }
        }
    }

    /// Unsubscribes a session by ID from an object key.
    ///
    /// Returns whether unsubscribing was successful. This will only be
    /// unsuccessful if the session wasn't subscribed to the key.
    pub fn unsubscribe(&self, object_key: &[u8], session_id: SessionId) -> bool {
        let key_unsubbed = self
            .0
            .keys
            .get(object_key)
            .and_then(|sessions| sessions.remove(&session_id))
            .is_some();
        let session_unsubbed = self
            .0
            .sessions
            .get(&session_id)
            .and_then(|keys| keys.remove(object_key))
            .is_some();

        // Really either both of these should be true or both false, but we'll
        // handle state inconsistencies elsewhere...
        key_unsubbed || session_unsubbed
    }

    /// Unsubscribes a session from all of its subscriptions.
    ///
    /// Returns whether the session was subscribed to any channels.
    pub fn remove_session(&self, session_id: SessionId) -> bool {
        self.0
            .sessions
            .remove(&session_id)
            .map(|(_, subscriptions)| {
                for (_, sub) in subscriptions.into_iter() {
                    sub.close();
                }
            })
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{PubSubManager, PubSubManagerRef};
    use core::fmt::Debug;
    use static_assertions::assert_impl_all;

    assert_impl_all!(PubSubManagerRef: Debug, Default);
    assert_impl_all!(PubSubManager: Clone, Debug, Default);
}
