use super::prelude::*;
use crate::state::object::Integer;

pub struct IncrementIntBy;

impl IncrementIntBy {
    pub fn increment(hop: &Hop, key: &[u8], amount: i64) -> Result<i64> {
        let mut int = hop
            .state()
            .typed_key::<Integer>(key)
            .ok_or(Error::KeyRetrieval)?;

        *int += amount;

        Ok(*int)
    }
}

impl Dispatch for IncrementIntBy {
    fn dispatch(hop: &Hop, req: &mut Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;
        let new = Self::increment(hop, key, 1)?;

        Ok(Response::from_int(new))
    }
}
