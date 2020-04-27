use super::prelude::*;

pub struct Append<'a> {
    pub(super) state: &'a State,
}

impl<'a> Command<'a> for Append<'a> {
    fn new(state: &'a State) -> Self {
        Self {
            state,
        }
    }

    fn dispatch(self, req: Request) -> Result<Response> {
        let key = req.arg(0).ok_or(Error::KeyRetrieval)?;
        let arg = req.arg(1).ok_or(Error::ArgumentRetrieval)?;

        let mut value = self.state.bytes(key).map_err(|_| Error::WrongType)?;
        value.extend_from_slice(arg);

        Ok(Response::from_usize(value.len()))
    }
}
