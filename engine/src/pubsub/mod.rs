mod key_update;
mod manager;

pub use self::{key_update::KeyUpdate, manager::PubSubManager};

use futures_intrusive::channel::shared::{self, Receiver, Sender};

#[derive(Debug)]
pub struct Subscription {
    rx: Receiver<KeyUpdate>,
    tx: Sender<KeyUpdate>,
}

impl Subscription {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn close(&self) {
        self.tx.close();
    }

    pub fn receiver(&self) -> Receiver<KeyUpdate> {
        self.rx.clone()
    }

    pub fn sender(&self) -> Sender<KeyUpdate> {
        self.tx.clone()
    }
}

impl Default for Subscription {
    fn default() -> Self {
        let (tx, rx) = shared::unbuffered_channel();

        Self { rx, tx }
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyUpdate, Subscription};
    use core::fmt::Debug;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Subscription: Debug);

    #[tokio::test]
    async fn test_sub_close() {
        let sub = Subscription::new();
        let update = KeyUpdate::Renamed { to: b"b".to_vec() };
        sub.close();
        assert!(sub.sender().send(update).await.is_err());
    }
}
