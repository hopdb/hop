use super::prelude::*;

pub struct DecrementIntBy<'a> {
    pub(super) state: &'a mut State,
}

impl DecrementIntBy<'_> {
    pub fn decrement(&mut self, key: &[u8], amount: i64) -> Result<i64>  {
        let mut int = self.state.int(key).map_err(|_| Error::KeyRetrieval)?;

        *int -= amount;

        Ok(*int)
    }
}

impl<'a> Command<'a> for DecrementIntBy<'a> {
    fn new(state: &'a mut State) -> Self {
        Self {
            state,
        }
    }

    fn dispatch(mut self, mut req: Request) -> Result<Response> {
        let new = self.decrement(req.key().unwrap(), 1)?;

        Ok(Response::from_int(new))
    }
}
