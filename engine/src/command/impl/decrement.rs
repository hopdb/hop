use super::{super::{DispatchError, DispatchResult, Dispatch, Request, Response}, decrement_by::DecrementBy};
use crate::Hop;

pub struct Decrement;

impl Dispatch for Decrement {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Response> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        DecrementBy::decrement(hop, key, req.key_type(), 1)
    }
}
