use super::super::{response, Dispatch, DispatchResult, Request};
use crate::Hop;
use alloc::vec::Vec;

pub struct Echo;

impl Dispatch for Echo {
    fn dispatch(_: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        match req.args(..) {
            Some(args) => response::write_list(resp, args),
            None => {
                let empty_slice: &[&[u8]] = &[];

                response::write_list(resp, empty_slice);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Echo;
    use crate::{
        command::{response, CommandId, Dispatch, Request},
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_input() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"hopdb".to_vec());

        let req = Request::new(CommandId::Echo, Some(args.clone()));

        let mut expected = Vec::new();
        let mut resp = Vec::new();

        assert!(Echo::dispatch(&hop, &req, &mut resp).is_ok());
        response::write_list(&mut expected, args.as_slice());
        assert_eq!(resp, expected);

        expected.clear();
        resp.clear();

        args.push(b"hop".to_vec());
        let req = Request::new(CommandId::Echo, Some(args.clone()));

        assert!(Echo::dispatch(&hop, &req, &mut resp).is_ok());
        response::write_list(&mut expected, args.as_slice());
        assert_eq!(resp, expected);
    }

    #[test]
    fn test_empty() {
        let hop = Hop::new();
        let req = Request::new(CommandId::Echo, None);

        let args: &[Vec<_>] = &[];

        let mut expected = Vec::new();
        let mut resp = Vec::new();

        assert!(Echo::dispatch(&hop, &req, &mut resp).is_ok());
        response::write_list(&mut expected, args);
        assert_eq!(resp, expected);
    }
}
