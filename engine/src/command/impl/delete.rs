use super::super::{Dispatch, DispatchError, DispatchResult, Request, Response};
use crate::Hop;
use alloc::vec::Vec;

pub struct Delete;

impl Dispatch for Delete {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        if req.key_type().is_some() {
            return Err(DispatchError::KeyTypeUnexpected);
        }

        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;
        let (k, _) = hop
            .state()
            .remove(key)
            .ok_or(DispatchError::PreconditionFailed)?;

        let response = Response::from(k);
        response.copy_to(resp);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Delete;
    use crate::{
        command::{request::RequestBuilder, CommandId, Dispatch, DispatchError, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::vec::Vec;

    fn builder(key_type: impl Into<Option<KeyType>>) -> RequestBuilder {
        let mut builder = RequestBuilder::new_with_key_type(CommandId::DecrementBy, key_type);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());

        builder
    }

    #[test]
    fn test_valid() {
        let req = builder(None).into_request();
        let mut resp = Vec::new();

        let hop = Hop::new();
        hop.state()
            .insert(b"foo".to_vec(), Value::Bytes([b'f', b'o', b'o'].to_vec()));

        assert!(Delete::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(b"foo".to_vec()).as_bytes());
        assert!(!hop.state().contains_key(b"foo"));
    }

    #[test]
    fn test_nonexistent() {
        let req = builder(None).into_request();
        let mut resp = Vec::new();

        let hop = Hop::new();

        assert!(matches!(
            Delete::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::PreconditionFailed)
        ));
    }

    #[test]
    fn test_key_type_specified() {
        let req = builder(KeyType::Bytes).into_request();
        let mut resp = Vec::new();

        let hop = Hop::new();

        assert!(matches!(
            Delete::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::KeyTypeUnexpected)
        ));
    }

    #[test]
    fn test_no_arguments() {
        let req = RequestBuilder::new(CommandId::Delete).into_request();
        let mut resp = Vec::new();

        let hop = Hop::new();
        assert!(matches!(
            Delete::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::KeyUnspecified)
        ));
    }
}
