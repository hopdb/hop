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
