use super::prelude::*;

pub struct Echo;

impl Dispatch for Echo {
    fn dispatch(_: &Hop, req: &mut Request) -> Result<Response> {
        Ok(req.flatten_args().unwrap_or_default().into())
    }
}
