use super::prelude::*;

pub struct Ping;

impl Command<'_> for Ping {
    fn new(_: &mut State) -> Self {
        Self
    }

    fn dispatch(self, _: Request) -> Result<Response> {
        Ok("Pong!".into())
    }
}
