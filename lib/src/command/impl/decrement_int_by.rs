use super::prelude::*;
use crate::state::object::Integer;

pub struct DecrementIntBy;

impl DecrementIntBy {
    pub fn decrement(hop: &Hop, key: &[u8], amount: i64) -> Result<i64> {
        let mut int = hop
            .state()
            .typed_key::<Integer>(key)
            .ok_or(Error::KeyRetrieval)?;

        *int -= amount;

        Ok(*int)
    }
}

impl Dispatch for DecrementIntBy {
    fn dispatch(hop: &Hop, req: &Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;
        let new = Self::decrement(hop, key, 1)?;

        Ok(Response::from(new))
    }
}
