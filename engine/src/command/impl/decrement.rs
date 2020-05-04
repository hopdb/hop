use super::{
    super::{Dispatch, DispatchError, DispatchResult, Request},
    decrement_by::DecrementBy,
};
use crate::Hop;
use alloc::vec::Vec;

pub struct Decrement;

impl Dispatch for Decrement {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Vec<u8>> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        DecrementBy::decrement(hop, key, req.key_type(), 1)
    }
}
