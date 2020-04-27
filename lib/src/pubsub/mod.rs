mod key_update;
mod manager;

pub use self::{
    key_update::KeyUpdate,
    manager::PubSubManager,
};

use futures_core::stream::Stream;
use futures_intrusive::channel::UnbufferedChannel;

#[derive(Debug)]
pub struct Subscription {
    channel: UnbufferedChannel<KeyUpdate>,
}

impl Subscription {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn stream(&self) -> impl Stream<Item = KeyUpdate> + '_ {
        self.channel.stream()
    }
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            channel: UnbufferedChannel::new(),
        }
    }
}
