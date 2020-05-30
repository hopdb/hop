use crate::{
    command::{response, Dispatch, DispatchError, DispatchResult, Request},
    Hop,
};
use alloc::vec::Vec;

pub struct Get;

impl Dispatch for Get {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;
        let state = hop.state();
        let r = state
            .key_ref(key)
            .ok_or_else(|| DispatchError::KeyNonexistent)?;

        if let Some(key_type) = req.key_type() {
            if r.value().kind() != key_type {
                return Err(DispatchError::KeyTypeDifferent);
            }
        }

        response::write_value(resp, r.value());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Get;
    use crate::{
        command::{request::RequestBuilder, CommandId, Dispatch, DispatchError, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_bool() {
        let mut builder = RequestBuilder::new(CommandId::Get);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let mut resp = Vec::new();
        let hop = Hop::new();
        hop.state().insert(b"foo".to_vec(), Value::Boolean(false));

        assert!(Get::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(false).as_bytes());
    }

    #[test]
    fn test_bool_specified_key_type() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Get, KeyType::Boolean);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let mut resp = Vec::new();
        let hop = Hop::new();
        hop.state().insert(b"foo".to_vec(), Value::Boolean(true));

        assert!(Get::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(true).as_bytes());
    }

    #[test]
    fn test_int() {
        let mut builder = RequestBuilder::new(CommandId::Get);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let mut resp = Vec::new();
        let hop = Hop::new();
        hop.state().insert(b"foo".to_vec(), Value::Integer(123));

        assert!(Get::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(123).as_bytes());
    }

    #[test]
    fn test_no_key() {
        let req = RequestBuilder::new(CommandId::Get).into_request();

        let mut resp = Vec::new();
        let hop = Hop::new();

        assert_eq!(
            DispatchError::KeyUnspecified,
            Get::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }

    #[test]
    fn test_key_nonexistent() {
        let mut builder = RequestBuilder::new(CommandId::Get);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let mut resp = Vec::new();
        let hop = Hop::new();

        assert_eq!(
            DispatchError::KeyNonexistent,
            Get::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }

    #[test]
    fn test_key_type_different() {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::Get, KeyType::Boolean);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let mut resp = Vec::new();
        let hop = Hop::new();
        hop.state().insert(b"foo".to_vec(), Value::Integer(123));

        assert_eq!(
            DispatchError::KeyTypeDifferent,
            Get::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }
}
