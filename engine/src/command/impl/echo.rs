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
        command::{request::RequestBuilder, response, CommandId, Dispatch},
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_input() {
        let hop = Hop::new();

        let mut builder = RequestBuilder::new(CommandId::Echo);
        assert!(builder.bytes(b"hopdb".as_ref()).is_ok());
        let req = builder.clone().into_request();

        let mut expected = Vec::new();
        let mut resp = Vec::new();

        assert!(Echo::dispatch(&hop, &req, &mut resp).is_ok());
        response::write_list(&mut expected, [b"hopdb"].as_ref());
        assert_eq!(resp, expected);

        expected.clear();
        resp.clear();

        assert!(builder.bytes(b"hop".as_ref()).is_ok());
        let req = builder.into_request();

        assert!(Echo::dispatch(&hop, &req, &mut resp).is_ok());
        response::write_list(&mut expected, [b"hopdb".as_ref(), b"hop".as_ref()].as_ref());
        assert_eq!(resp, expected);
    }

    #[test]
    fn test_empty() {
        let req = RequestBuilder::new(CommandId::Echo).into_request();

        let hop = Hop::new();
        let empty_slice: &[Vec<_>] = &[];

        let mut expected = Vec::new();
        let mut resp = Vec::new();

        assert!(Echo::dispatch(&hop, &req, &mut resp).is_ok());
        response::write_list(&mut expected, empty_slice);
        assert_eq!(resp, expected);
    }
}
