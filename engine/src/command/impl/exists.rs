use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::Hop;
use alloc::vec::Vec;

pub struct Exists;

impl Dispatch for Exists {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        if req.key_type().is_some() {
            return Err(DispatchError::KeyTypeUnexpected);
        }

        let args = req.args(..).ok_or(DispatchError::ArgumentRetrieval)?;
        let state = hop.state();

        let all = args.iter().all(|key| state.contains_key(key));

        response::write_bool(resp, all);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Exists;
    use crate::{
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::vec::Vec;

    fn args() -> Vec<Vec<u8>> {
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push(b"bar".to_vec());

        args
    }

    #[test]
    fn test_one_key() {
        let mut args = args();
        args.pop();
        let req = Request::new(CommandId::Exists, Some(args));
        let mut resp = Vec::new();

        let hop = Hop::new();
        hop.state()
            .insert(b"foo".to_vec(), Value::Bytes([1, 2, 3].to_vec()));

        assert!(Exists::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(true).as_bytes());
    }

    #[test]
    fn test_two_keys_both_exist() {
        let args = args();
        let req = Request::new(CommandId::Exists, Some(args));
        let mut resp = Vec::new();

        let hop = Hop::new();
        hop.state()
            .insert(b"foo".to_vec(), Value::Bytes([1, 2, 3].to_vec()));
        hop.state()
            .insert(b"bar".to_vec(), Value::Bytes([1, 2, 3].to_vec()));

        assert!(Exists::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(true).as_bytes());
    }

    #[test]
    fn test_two_keys_one_exists() {
        let args = args();
        let req = Request::new(CommandId::Exists, Some(args));
        let mut resp = Vec::new();

        let hop = Hop::new();
        hop.state()
            .insert(b"foo".to_vec(), Value::Bytes([1, 2, 3].to_vec()));

        assert!(Exists::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(false).as_bytes());
    }

    #[test]
    fn test_one_key_doesnt_exist() {
        let mut args = args();
        args.pop();
        let req = Request::new(CommandId::Exists, Some(args));
        let mut resp = Vec::new();

        let hop = Hop::new();

        assert!(Exists::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(false).as_bytes());
    }

    #[test]
    fn test_no_arguments() {
        let req = Request::new(CommandId::Exists, None);
        let mut resp = Vec::new();

        let hop = Hop::new();
        assert!(matches!(
            Exists::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::ArgumentRetrieval)
        ));
    }

    #[test]
    fn test_key_type_specified() {
        let req = Request::new_with_type(CommandId::Exists, Some(args()), KeyType::List);
        let mut resp = Vec::new();

        let hop = Hop::new();
        assert!(matches!(
            Exists::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::KeyTypeUnexpected)
        ));
    }
}
