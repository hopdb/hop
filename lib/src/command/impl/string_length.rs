use super::prelude::*;
use crate::state::object::Str;

pub struct StringLength;

impl Dispatch for StringLength {
    fn dispatch(hop: &Hop, req: &mut Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;
        let string = match hop.state().typed_key::<Str>(key) {
            Some(string) => string,
            None => return Ok(Response::from_usize(0)),
        };

        Ok(Response::from_usize(string.len()))
    }
}
