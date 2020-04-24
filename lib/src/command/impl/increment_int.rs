use super::{
    increment_int_by::IncrementIntBy,
    prelude::*,
};

pub struct IncrementInt<'a> {
    state: &'a mut State,
}

impl<'a> Command<'a> for IncrementInt<'a> {
    fn new(state: &'a mut State) -> Self {
        Self {
            state,
        }
    }

    fn dispatch(self, mut req: Request) -> Result<Response> {
        let mut inner = IncrementIntBy {
            state: self.state,
        };

        let new = inner.increment(req.key().unwrap(), 1)?;

        Ok(Response::from_int(new))
    }
}
