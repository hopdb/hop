use super::prelude::*;

pub struct Stats;

impl Dispatch for Stats {
    fn dispatch(_: &Hop, _: &Request) -> Result<Response> {
        Ok("Pong!".into())
    }
}
