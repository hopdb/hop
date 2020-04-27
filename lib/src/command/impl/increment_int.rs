use super::{
    increment_int_by::IncrementIntBy,
    prelude::*,
};

pub struct IncrementInt<'a> {
    state: &'a State,
}

impl<'a> Command<'a> for IncrementInt<'a> {
    fn new(state: &'a State) -> Self {
        Self {
            state,
        }
    }

    fn dispatch(self, mut req: Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;
        let inner = IncrementIntBy {
            state: self.state,
        };

        let new = inner.increment(key, 1)?;

        Ok(Response::from_int(new))
    }
}
