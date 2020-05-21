use super::{
    super::{Dispatch, DispatchError, DispatchResult, Request},
    increment_by::IncrementBy,
};
use crate::Hop;
use alloc::vec::Vec;

pub struct Increment;

impl Dispatch for Increment {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        IncrementBy::increment(hop, key, req.key_type(), 1, resp)
    }
}
