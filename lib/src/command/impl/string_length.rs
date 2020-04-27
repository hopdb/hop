use super::prelude::*;
use crate::state::object::Object;

pub struct StringLength<'a> {
    state: &'a State,
}

impl<'a> Command<'a> for StringLength<'a> {
    fn new(state: &'a State) -> Self {
        Self { state }
    }

    fn dispatch(self, mut req: Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;
        let value = match self.state.key_optional(key) {
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
