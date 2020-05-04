use super::super::{DispatchResult, Dispatch, Request, response};
use alloc::vec::Vec;
use crate::Hop;

pub struct Stats;

impl Dispatch for Stats {
    fn dispatch(_: &Hop, _: &Request) -> DispatchResult<Vec<u8>> {
        Ok(response::write_str("Pong!"))
    }
}
