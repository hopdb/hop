use super::prelude::*;

pub struct Ping;

impl Dispatch for Ping {
    fn dispatch(_: &Hop, _: &mut Request) -> Result<Response> {
        Ok("Pong!".into())
    }
}
