use super::{super::{DispatchError, DispatchResult, Dispatch, Request}, increment_by::IncrementBy};
use alloc::vec::Vec;
use crate::Hop;

pub struct Increment;

impl Dispatch for Increment {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Vec<u8>> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        IncrementBy::increment(hop, key, req.key_type(), 1)
    }
}
