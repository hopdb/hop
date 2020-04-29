use super::prelude::*;
use alloc::vec::Vec;

pub struct Echo;

impl Dispatch for Echo {
    fn dispatch(_: &Hop, req: &mut Request) -> Result<Response> {
        match req.args() {
            Some(args) => Ok(Response::from(args)),
            None => {
                // The type system isn't able to reason about the type of the
                // slice when doing something like
                // `Response::from([].as_ref())`.
                let empty_slice: &[Vec<_>] = &[];

                Ok(Response::from(empty_slice))
            }
        }
    }
}
