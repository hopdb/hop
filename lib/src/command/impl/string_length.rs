use super::prelude::*;
use crate::state::object::Object;

pub struct StringLength;

impl Dispatch for StringLength {
    fn dispatch(hop: &Hop, req: &mut Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;
        let value = match hop.state().key_optional(key) {
            Some(value) => value,
            None => return Ok(Response::from_usize(0)),
        };

        let s = match value.value() {
            Object::String(string) => string,
            _ => return Err(Error::WrongType),
        };

        Ok(Response::from_usize(s.len()))
    }
}
