use super::{decrement_by::DecrementBy, prelude::*};

pub struct Decrement;

impl Dispatch for Decrement {
    fn dispatch(hop: &Hop, req: &Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;

        DecrementBy::decrement(hop, key, req.key_type(), 1)
    }
}
