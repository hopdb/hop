use super::super::{DispatchResult, Dispatch, Request, Response};
use alloc::vec::Vec;
use crate::Hop;

pub struct Echo;

impl Dispatch for Echo {
    fn dispatch(_: &Hop, req: &Request) -> DispatchResult<Response> {
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

#[cfg(test)]
mod tests {
    use super::Echo;
    use crate::{
        command::{CommandId, Dispatch, Request, Response},
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_input() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"hopdb".to_vec());

        let req = Request::new(CommandId::Echo, Some(args.clone()));

        assert_eq!(
            Echo::dispatch(&hop, &req).unwrap().into_bytes(),
            Response::from(args.as_slice()).into_bytes()
        );

        args.push(b"hop".to_vec());

        let req = Request::new(CommandId::Echo, Some(args.clone()));
        assert_eq!(
            Echo::dispatch(&hop, &req).unwrap().into_bytes(),
            Response::from(args.as_slice()).into_bytes()
        );
    }

    #[test]
    fn test_empty() {
        let hop = Hop::new();
        let req = Request::new(CommandId::Echo, None);

        let args: &[Vec<_>] = &[];
        assert_eq!(
            Echo::dispatch(&hop, &req).unwrap().into_bytes(),
            Response::from(args).into_bytes()
        );
    }
}
