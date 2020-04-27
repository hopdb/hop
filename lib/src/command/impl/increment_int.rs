use super::{increment_int_by::IncrementIntBy, prelude::*};

pub struct IncrementInt;

impl Dispatch for IncrementInt {
    fn dispatch(hop: &Hop, req: &mut Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;

        let new = IncrementIntBy::increment(hop, key, 1)?;

        Ok(Response::from_int(new))
    }
}
