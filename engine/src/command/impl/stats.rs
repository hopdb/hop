use super::super::{DispatchResult, Dispatch, Request, Response};
use crate::Hop;

pub struct Stats;

impl Dispatch for Stats {
    fn dispatch(_: &Hop, _: &Request) -> DispatchResult<Response> {
        Ok("Pong!".into())
    }
}
