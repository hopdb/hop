use super::prelude::*;

pub struct IncrementIntBy<'a> {
    pub(super) state: &'a State,
}

impl IncrementIntBy<'_> {
    pub fn increment(&self, key: &[u8], amount: i64) -> Result<i64>  {
        let mut int = self.state.int(key).map_err(|_| Error::KeyRetrieval)?;

        *int += amount;

        Ok(*int)
    }
}

impl<'a> Command<'a> for IncrementIntBy<'a> {
    fn new(state: &'a State) -> Self {
        Self {
            state,
        }
    }

    fn dispatch(self, mut req: Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;
        let new = self.increment(key, 1)?;

        Ok(Response::from_int(new))
    }
}
