use super::{decrement_int_by::DecrementIntBy, prelude::*};

pub struct DecrementInt;

impl Dispatch for DecrementInt {
    fn dispatch(hop: &Hop, req: &Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;

        let new = DecrementIntBy::decrement(hop, key, 1)?;

        Ok(Response::from(new))
    }
}
