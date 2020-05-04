use super::{super::{DispatchError, DispatchResult, Dispatch, Request, Response}, increment_by::IncrementBy};
use crate::Hop;

pub struct Increment;

impl Dispatch for Increment {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Response> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        IncrementBy::increment(hop, key, req.key_type(), 1)
    }
}
