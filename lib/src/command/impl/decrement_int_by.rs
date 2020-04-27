use super::prelude::*;

pub struct DecrementIntBy<'a> {
    pub(super) state: &'a State,
}

impl DecrementIntBy<'_> {
    pub fn decrement(&self, key: &[u8], amount: i64) -> Result<i64> {
        let mut int = self.state.int(key).map_err(|_| Error::KeyRetrieval)?;

        *int -= amount;

        Ok(*int)
    }
}

impl<'a> Command<'a> for DecrementIntBy<'a> {
    fn new(state: &'a State) -> Self {
        Self { state }
    }

    fn dispatch(self, mut req: Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;
        let new = self.decrement(key, 1)?;

        Ok(Response::from_int(new))
    }
}
