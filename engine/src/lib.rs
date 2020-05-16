#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod command;
pub mod metrics;
pub mod pubsub;
pub mod session;
pub mod state;

mod pool;

use self::{
    command::{r#impl::*, CommandId, Dispatch, DispatchResult, Request},
    metrics::{Metric, Metrics, Reader, Writer},
    pubsub::PubSubManager,
    session::SessionManager,
    state::State,
};
use alloc::{sync::Arc, vec::Vec};

#[derive(Debug)]
struct HopRef {
    metrics: Metrics,
    metrics_writer: Writer,
    pubsub: PubSubManager,
    sessions: SessionManager,
    state: State,
}

#[derive(Clone, Debug)]
pub struct Hop(Arc<HopRef>);

impl Hop {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn dispatch(&self, req: &Request, res: &mut Vec<u8>) -> DispatchResult<()> {
        let res = match req.kind() {
            CommandId::Append => Append::dispatch(self, req, res),
            CommandId::DecrementBy => DecrementBy::dispatch(self, req, res),
            CommandId::Decrement => Decrement::dispatch(self, req, res),
            CommandId::Echo => Echo::dispatch(self, req, res),
            CommandId::Increment => Increment::dispatch(self, req, res),
            CommandId::IncrementBy => IncrementBy::dispatch(self, req, res),
            CommandId::Stats => Stats::dispatch(self, req, res),
            CommandId::Length => Length::dispatch(self, req, res),
        };

        self.0.metrics_writer.increment(if res.is_ok() {
            Metric::CommandsSuccessful
        } else {
            Metric::CommandsErrored
        });

        res
    }

    pub fn metrics(&self) -> Reader {
        self.0.metrics.reader()
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

impl Default for Hop {
    fn default() -> Self {
        let metrics = Metrics::default();
        let writer = metrics.writer();

        Self(Arc::new(HopRef {
            metrics,
            metrics_writer: writer.clone(),
            pubsub: PubSubManager::default(),
            sessions: SessionManager::new(writer),
            state: State::default(),
        }))
    }
}
