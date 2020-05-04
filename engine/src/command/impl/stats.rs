use super::super::{response, Dispatch, DispatchResult, Request};
use crate::Hop;
use alloc::vec::Vec;

pub struct Stats;

impl Dispatch for Stats {
    fn dispatch(_: &Hop, _: &Request) -> DispatchResult<Vec<u8>> {
        Ok(response::write_str("Pong!"))
    }
}
