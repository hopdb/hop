use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::Hop;
use alloc::vec::Vec;

pub struct Rename;

impl Dispatch for Rename {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        if req.key_type().is_some() {
            return Err(DispatchError::KeyTypeUnexpected);
        }

        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;
        let arg = req.arg(1).ok_or(DispatchError::ArgumentRetrieval)?;
        let state = hop.state();

        if !state.contains_key(key) {
            return Err(DispatchError::KeyNonexistent);
        }

        if state.contains_key(arg) {
            return Err(DispatchError::PreconditionFailed);
        }

        let (_, v) = state.remove(key).ok_or(DispatchError::KeyNonexistent)?;
        state.insert(arg.to_vec(), v);

        response::write_bytes(resp, arg);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Rename;
    use crate::{
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::Value,
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
    fn test_rename_valid() {
        let args = args();
        let req = Request::new(CommandId::Rename, Some(args));
        let mut resp = Vec::new();

        let hop = Hop::new();
        hop.state()
            .insert(b"foo".to_vec(), Value::Bytes([1, 2, 3].to_vec()));

        assert!(Rename::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(b"bar".to_vec()).as_bytes());
    }

    #[test]
    fn test_rename_src_nonexistent() {
        let args = args();
        let req = Request::new(CommandId::Rename, Some(args));
        let mut resp = Vec::new();

        let hop = Hop::new();

        assert!(matches!(
            Rename::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::KeyNonexistent)
        ));
    }

    #[test]
    fn test_rename_destination_already_exists() {
        let args = args();
        let req = Request::new(CommandId::Rename, Some(args));
        let mut resp = Vec::new();

        let hop = Hop::new();
        hop.state().insert(b"foo".to_vec(), Value::bytes());
        hop.state().insert(b"bar".to_vec(), Value::bytes());

        assert!(matches!(
            Rename::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::PreconditionFailed)
        ));
    }

    #[test]
    fn test_too_few_arguments() {
        let mut args = args();
        args.pop();
        let req = Request::new(CommandId::Rename, Some(args.clone()));
        let mut resp = Vec::new();

        let hop = Hop::new();
        assert!(matches!(
            Rename::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::ArgumentRetrieval)
        ));

        args.pop();
        let req = Request::new(CommandId::Rename, Some(args));
        resp.clear();

        assert!(matches!(
            Rename::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::KeyUnspecified)
        ));
    }
}
