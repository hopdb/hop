use super::{
    super::{Dispatch, DispatchError, DispatchResult, Request},
    decrement_by::DecrementBy,
};
use crate::Hop;
use alloc::vec::Vec;

pub struct Decrement;

impl Dispatch for Decrement {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        DecrementBy::decrement(hop, key, req.key_type(), 1, resp)
    }
}
