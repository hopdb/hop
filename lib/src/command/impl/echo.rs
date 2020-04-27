use super::prelude::*;

pub struct Echo;

impl Command<'_> for Echo {
    fn new(_: &State) -> Self {
        Self
    }

    fn dispatch(self, req: Request) -> Result<Response> {
        Ok(req.flatten_args().into())
    }
}
