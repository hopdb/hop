use crate::state::{Key, Value};

pub enum KeyUpdate {
    /// The value of the subscribed key was deleted, meaning that the key did
    /// have a value but no longer does.
    Deleted(Value),
    /// The value of the subscribed key was initialized, meaning that the key
    /// didn't have a value but now does.
    Initialized(Value),
    /// The value of the key was moved to a different key.
    ///
    /// The key that the value was moved to is provided.
    ///
    /// The subscription itself does not move to the new key. Clients must
    /// subscribe to the new key and/or unsubscribe from the original if
    /// they want.
    Renamed {
        /// The key that the value was moved to.
        to: Key,
    },
    /// The value of the subscribed key was updated.
    Updated(Value),
}
