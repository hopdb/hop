use super::{super::{DispatchError, DispatchResult, Dispatch, Request}, decrement_by::DecrementBy};
use alloc::vec::Vec;
use crate::Hop;

pub struct Decrement;

impl Dispatch for Decrement {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Vec<u8>> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        DecrementBy::decrement(hop, key, req.key_type(), 1)
    }
}
