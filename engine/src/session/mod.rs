mod manager;

pub use manager::SessionManager;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SessionId(u32);

#[derive(Debug)]
pub struct Session {
    pub client_token: [u8; 32],
    pub id: u32,
}

impl Session {
    pub fn new(client_token: [u8; 32], id: u32) -> Self {
        Self { client_token, id }
    }
}

#[cfg(test)]
mod tests {
    use super::{Session, SessionId};
    use core::{fmt::Debug, hash::Hash};
    use static_assertions::assert_impl_all;

    assert_impl_all!(
        SessionId: Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    );
    assert_impl_all!(Session: Debug);
}
