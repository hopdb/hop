use super::Session;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};
use dashmap::{
    mapref::{entry::Entry, one::RefMut},
    DashMap,
};

#[derive(Debug, Default)]
struct SessionManagerRef {
    next_id: AtomicU32,
    sessions: DashMap<u32, Session>,
}

#[derive(Clone, Debug, Default)]
pub struct SessionManager(Arc<SessionManagerRef>);

impl SessionManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, client_token: [u8; 32]) -> RefMut<'_, u32, Session> {
        let session = Session::new(client_token, self.next_id());

        let value = match self.0.sessions.entry(self.next_id()) {
            Entry::Occupied(_) => {
                unreachable!();
            }
            Entry::Vacant(v) => v.insert(session),
        };

        self.0
            .next_id
            .store(self.next_id().wrapping_add(1), Ordering::SeqCst);

        value
    }

    pub fn remove(&self, id: u32) -> bool {
        self.0.sessions.remove(&id).is_some()
    }

    fn next_id(&self) -> u32 {
        self.0.next_id.load(Ordering::SeqCst)
    }
}
