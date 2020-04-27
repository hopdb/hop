use super::{decrement_int_by::DecrementIntBy, prelude::*};

pub struct DecrementInt<'a> {
    state: &'a State,
}

impl<'a> Command<'a> for DecrementInt<'a> {
    fn new(state: &'a State) -> Self {
        Self { state }
    }

    fn dispatch(self, mut req: Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;

        let inner = DecrementIntBy { state: self.state };

        let new = inner.decrement(key, 1)?;

        Ok(Response::from_int(new))
    }
}
