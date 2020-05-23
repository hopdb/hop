//! Utilities for constructing and using an instance of the Hop engine.
//!
//! Included is a [`Config`] which holds the configuration running a Hop engine,
//! the [`Hop`] engine itself which can be used to dispatch commands and
//! retrieve information about the engine, and a [`Builder`] for constructing a
//! user-configured instance of the engine.
//!
//! [`Builder`]: struct.Builder.html
//! [`Config`]: struct.Config.html
//! [`Hop`]: struct.Hop.html

use crate::{
    command::{r#impl::*, CommandId, Dispatch, DispatchResult, Request},
    metrics::{Metric, Metrics, Reader, Writer},
    pubsub::PubSubManager,
    session::SessionManager,
    state::State,
};
use alloc::{sync::Arc, vec::Vec};

/// Configuration defining how a Hop engine will operate.
///
/// This includes things like enabling or disabling pubsub support.
///
/// See [`Builder`] for constructing a configured Hop engine.
///
/// [`Builder`]: struct.Builder.html
#[derive(Clone, Debug)]
pub struct Config {
    pubsub_enabled: bool,
    sessions_active_max: usize,
}

impl Config {
    /// Retrieve whether pubsub is enabled.
    pub fn pubsub_enabled(&self) -> bool {
        self.pubsub_enabled
    }

    /// Retrieve the maximum number of active sessions that are allowed at a
    /// time.
    pub fn sessions_active_max(&self) -> usize {
        self.sessions_active_max
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pubsub_enabled: true,
            sessions_active_max: usize::MAX,
        }
    }
}

/// A builder to construct a configured [`Hop`] engine instance.
///
/// Refer to each method for its default value.
///
/// # Examples
///
/// Construct a Hop engine with pubsub disabled and a maximum of 150 active
/// sessions at a time:
///
/// ```
/// use hop_engine::hop::Builder;
///
/// let mut builder = Builder::new();
/// builder.pubsub_enabled(false).sessions_active_max(150);
/// let hop = builder.build();
///
/// assert!(!hop.config().pubsub_enabled());
/// assert_eq!(150, hop.config().sessions_active_max());
/// ```
///
/// [`Hop`]: struct.Hop.html
#[derive(Clone, Debug, Default)]
pub struct Builder(Config);

impl Builder {
    /// Create a new builder with the default values of a [`Hop`] instance.
    ///
    /// [`Hop`]: struct.Hop.html
    pub fn new() -> Self {
        Self::default()
    }

    /// Consume the builder and construct a `Hop` instance.
    pub fn build(self) -> Hop {
        self.into()
    }

    /// Set whether to enable pubsub.
    ///
    /// By default this is `true`.
    pub fn pubsub_enabled(&mut self, pubsub_enabled: bool) -> &mut Self {
        self.0.pubsub_enabled = pubsub_enabled;

        self
    }

    /// Set whether to enable pubsub.
    ///
    /// By default this is the maximum usize value.
    pub fn sessions_active_max(&mut self, sessions_active_max: usize) -> &mut Self {
        self.0.sessions_active_max = sessions_active_max;

        self
    }
}

impl From<Builder> for Hop {
    fn from(builder: Builder) -> Self {
        Self(Arc::new(HopRef {
            config: builder.0,
            ..Default::default()
        }))
    }
}

#[derive(Debug)]
pub(crate) struct HopRef {
    config: Config,
    metrics: Metrics,
    pub(crate) metrics_writer: Writer,
    pubsub: PubSubManager,
    sessions: SessionManager,
    state: State,
}

impl Default for HopRef {
    fn default() -> Self {
        let metrics = Metrics::default();
        let writer = metrics.writer();

        Self {
            config: Config::default(),
            metrics,
            metrics_writer: writer.clone(),
            pubsub: PubSubManager::default(),
            sessions: SessionManager::new(writer),
            state: State::default(),
        }
    }
}

/// The hop engine.
#[derive(Clone, Debug, Default)]
pub struct Hop(pub(crate) Arc<HopRef>);

impl Hop {
    /// Create a new instance of the engine using the default configuration.
    ///
    /// Refer to the [`builder`] method if you want to customise the engine.
    ///
    /// [`builder`]: #method.builder
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new builder for constructing a configured engine.
    ///
    /// Refer to the `Builder` documentation for more information.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Dispatch a request to the engine, providing a response to write the
    /// response to on success.
    pub fn dispatch(&self, req: &Request, res: &mut Vec<u8>) -> DispatchResult<()> {
        let res = match req.kind() {
            CommandId::Append => Append::dispatch(self, req, res),
            CommandId::DecrementBy => DecrementBy::dispatch(self, req, res),
            CommandId::Decrement => Decrement::dispatch(self, req, res),
            CommandId::Delete => Delete::dispatch(self, req, res),
            CommandId::Echo => Echo::dispatch(self, req, res),
            CommandId::Exists => Exists::dispatch(self, req, res),
            CommandId::Increment => Increment::dispatch(self, req, res),
            CommandId::IncrementBy => IncrementBy::dispatch(self, req, res),
            CommandId::Is => Is::dispatch(self, req, res),
            CommandId::Rename => Rename::dispatch(self, req, res),
            CommandId::Set => Set::dispatch(self, req, res),
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

    /// Return an immutable reference to the configuration.
    pub fn config(&self) -> &Config {
        &self.0.config
    }

    /// Return a new reader to read metrics from.
    pub fn metrics(&self) -> Reader {
        self.0.metrics.reader()
    }

    /// Return an immutable reference to the pubsub manager.
    pub fn pubsub(&self) -> &PubSubManager {
        &self.0.pubsub
    }

    /// Return an immutable reference to the session manager.
    pub fn sessions(&self) -> &SessionManager {
        &self.0.sessions
    }

    /// Return an immutable reference to the state.
    pub fn state(&self) -> &State {
        &self.0.state
    }
}

#[cfg(test)]
mod tests {
    use super::{Hop, HopRef};
    use core::fmt::Debug;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Hop: Clone, Debug, Default);
    assert_impl_all!(HopRef: Debug);
}
