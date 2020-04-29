#![no_std]

extern crate alloc;

pub mod command;
pub mod pubsub;
pub mod session;
pub mod state;

mod pool;

use self::{
    command::{r#impl::*, CommandResult, CommandType, Dispatch, Request, Response},
    pubsub::PubSubManager,
    session::SessionManager,
    state::State,
};
use alloc::sync::Arc;

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

    pub fn dispatch(&self, req: &Request) -> CommandResult<Response> {
        match req.kind() {
            CommandType::Append => Append::dispatch(self, req),
            CommandType::DecrementIntBy => DecrementIntBy::dispatch(self, req),
            CommandType::DecrementInt => DecrementInt::dispatch(self, req),
            CommandType::Echo => Echo::dispatch(self, req),
            CommandType::IncrementInt => IncrementInt::dispatch(self, req),
            CommandType::IncrementIntBy => IncrementIntBy::dispatch(self, req),
            CommandType::Stats => Stats::dispatch(self, req),
            CommandType::StringLength => StringLength::dispatch(self, req),
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
