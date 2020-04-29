use super::prelude::*;
use crate::state::object::Str;

pub struct StringLength;

impl Dispatch for StringLength {
    fn dispatch(hop: &Hop, req: &Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;
        let string = match hop.state().typed_key::<Str>(key) {
            Some(string) => string,
            None => return Ok(Response::from(0i64)),
        };

        Ok(Response::from(string.len() as i64))
    }
}
