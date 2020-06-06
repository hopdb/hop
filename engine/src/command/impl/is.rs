use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::Hop;
use alloc::vec::Vec;

pub struct Is;

impl Dispatch for Is {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key_type = req
            .key_type()
            .ok_or_else(|| DispatchError::KeyTypeRequired)?;
        let mut args = req.args(..).ok_or(DispatchError::ArgumentRetrieval)?;

        let all = args.all(|key| {
            hop.state()
                .key_type(key)
                .map(|k| k == key_type)
                .unwrap_or(false)
        });

        response::write_bool(resp, all);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Is;
    use crate::{
        command::{request::RequestBuilder, CommandId, Dispatch, DispatchError, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_one_arg() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Is, KeyType::String);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();
        hop.state().key_or_insert_with(b"foo", Value::string);

        let mut resp = Vec::new();

        assert!(Is::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(true).as_bytes());
    }

    #[test]
    fn test_two_args() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Is, KeyType::String);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(b"bar".as_ref()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();
        hop.state().key_or_insert_with(b"foo", Value::string);
        hop.state().key_or_insert_with(b"bar", Value::string);

        let mut resp = Vec::new();

        assert!(Is::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(true).as_bytes());
    }

    #[test]
    fn test_two_mismatched() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Is, KeyType::String);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.bytes(b"bar".as_ref()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();
        hop.state().key_or_insert_with(b"foo", Value::string);
        hop.state().key_or_insert_with(b"bar", Value::integer);

        let mut resp = Vec::new();

        assert!(Is::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(false).as_bytes());
    }

    #[test]
    fn test_no_arguments() {
        let builder = RequestBuilder::new_with_key_type(CommandId::Is, KeyType::Bytes);
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(matches!(
            Is::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::ArgumentRetrieval)
        ));
    }

    #[test]
    fn test_key_type_unspecified() {
        let mut builder = RequestBuilder::new(CommandId::Is);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();

        let mut resp = Vec::new();

        assert!(matches!(
            Is::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::KeyTypeRequired)
        ));
    }
}
