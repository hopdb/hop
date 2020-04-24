use super::{
    decrement_int_by::DecrementIntBy,
    prelude::*,
};

pub struct DecrementInt<'a> {
    state: &'a mut State,
}

impl<'a> Command<'a> for DecrementInt<'a> {
    fn new(state: &'a mut State) -> Self {
        Self {
            state,
        }
    }

    fn dispatch(self, mut req: Request) -> Result<Response> {
        let mut inner = DecrementIntBy {
            state: self.state,
        };

        let new = inner.decrement(req.key().unwrap(), 1)?;

        Ok(Response::from_int(new))
    }
}
