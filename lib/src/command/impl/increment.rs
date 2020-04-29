use super::{increment_by::IncrementBy, prelude::*};

pub struct Increment;

impl Dispatch for Increment {
    fn dispatch(hop: &Hop, req: &Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;

        IncrementBy::increment(hop, key, req.key_type(), 1)
    }
}
