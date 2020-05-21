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
        let state = hop.state();

        let (k, _) = state.remove(key).ok_or(DispatchError::PreconditionFailed)?;

        let response = Response::from(k);
        response.copy_to(resp);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Delete;
    use crate::{
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::vec::Vec;

    fn args() -> Vec<Vec<u8>> {
        let mut args = Vec::new();
        args.push(b"foo".to_vec());

        args
    }

    #[test]
    fn test_valid() {
        let args = args();
        let req = Request::new(CommandId::Delete, Some(args));
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
        let args = args();
        let req = Request::new(CommandId::Delete, Some(args));
        let mut resp = Vec::new();

        let hop = Hop::new();

        assert!(matches!(
            Delete::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::PreconditionFailed)
        ));
    }

    #[test]
    fn test_key_type_specified() {
        let args = args();
        let req = Request::new_with_type(CommandId::Delete, Some(args), KeyType::Bytes);
        let mut resp = Vec::new();

        let hop = Hop::new();

        assert!(matches!(
            Delete::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::KeyTypeUnexpected)
        ));
    }

    #[test]
    fn test_no_arguments() {
        let req = Request::new(CommandId::Delete, None);
        let mut resp = Vec::new();

        let hop = Hop::new();
        assert!(matches!(
            Delete::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::KeyUnspecified)
        ));
    }
}
