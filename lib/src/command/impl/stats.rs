use super::prelude::*;

pub struct Stats<'a> {
    state: &'a State,
}

impl<'a> Command<'a> for Stats<'a> {
    fn new(state: &'a State) -> Self {
        Self { state }
    }

    fn dispatch(self, _: Request) -> Result<Response> {
        Ok("Pong!".into())
    }
}
