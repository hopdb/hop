use imms::{
    protocol::Command,
    state::State as RawState,
};

/// A sealing state allowing only creation and dispatching.
///
/// There's no reason to use the other public methods that the library offers.
pub struct State {
    inner: RawState,
}

impl State {
    #[inline]
    pub fn new() -> Self {
        Self { inner: Default::default() }
    }

    #[inline]
    pub fn dispatch(&mut self, command: Command) -> bool {
        self.inner.dispatch(command)
    }
}
