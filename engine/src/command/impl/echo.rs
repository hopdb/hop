use super::super::{DispatchResult, Dispatch, Request, response};
use alloc::vec::Vec;
use crate::Hop;

pub struct Echo;

impl Dispatch for Echo {
    fn dispatch(_: &Hop, req: &Request) -> DispatchResult<Vec<u8>> {
        match req.args() {
            Some(args) => Ok(response::write_list(args)),
            None => Ok(response::write_list(&[])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Echo;
    use crate::{
        command::{CommandId, Dispatch, Request, response},
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
            Echo::dispatch(&hop, &req).unwrap(),
            response::write_list(args.as_slice()),
        );

        args.push(b"hop".to_vec());

        let req = Request::new(CommandId::Echo, Some(args.clone()));
        assert_eq!(
            Echo::dispatch(&hop, &req).unwrap(),
            response::write_list(args.as_slice()),
        );
    }

    #[test]
    fn test_empty() {
        let hop = Hop::new();
        let req = Request::new(CommandId::Echo, None);

        let args: &[Vec<_>] = &[];
        assert_eq!(
            Echo::dispatch(&hop, &req).unwrap(),
            response::write_list(args),
        );
    }
}
