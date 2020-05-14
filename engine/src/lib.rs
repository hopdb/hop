#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod command;
pub mod pubsub;
pub mod session;
pub mod state;

mod pool;

use self::{
    command::{r#impl::*, CommandId, Dispatch, DispatchResult, Request},
    pubsub::PubSubManager,
    session::SessionManager,
    state::State,
};
use alloc::{sync::Arc, vec::Vec};

#[derive(Debug, Default)]
struct HopRef {
    pubsub: PubSubManager,
    sessions: SessionManager,
    state: State,
}

#[derive(Clone, Debug, Default)]
pub struct Hop(Arc<HopRef>);

impl Hop {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn dispatch(&self, req: &Request) -> DispatchResult<Vec<u8>> {
        match req.kind() {
            CommandId::Append => Append::dispatch(self, req),
            CommandId::DecrementBy => DecrementBy::dispatch(self, req),
            CommandId::Decrement => Decrement::dispatch(self, req),
            CommandId::Echo => Echo::dispatch(self, req),
            CommandId::Increment => Increment::dispatch(self, req),
            CommandId::IncrementBy => IncrementBy::dispatch(self, req),
            CommandId::Stats => Stats::dispatch(self, req),
            CommandId::Length => Length::dispatch(self, req),
        }
    }

    pub fn pubsub(&self) -> &PubSubManager {
        &self.0.pubsub
    }

    pub fn sessions(&self) -> &SessionManager {
        &self.0.sessions
    }

    pub fn state(&self) -> &State {
        &self.0.state
    }
}
