use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::Hop;
use alloc::vec::Vec;

pub struct Is;

impl Dispatch for Is {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key_type = req
            .key_type()
            .ok_or_else(|| DispatchError::KeyTypeRequired)?;
        let args = req.args(..).ok_or(DispatchError::ArgumentRetrieval)?;
        let state = hop.state();

        let all = args.iter().all(|key| match state.key_ref(key) {
            Some(value) => value.value().kind() == key_type,
            None => false,
        });

        response::write_bool(resp, all);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Is;
    use crate::{
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_one_arg() {
        let hop = Hop::new();
        let mut args = Vec::new();
        hop.state().key_or_insert_with(b"foo", Value::string);
        args.push(b"foo".to_vec());

        let req = Request::new_with_type(CommandId::Is, Some(args), KeyType::String);
        let mut resp = Vec::new();

        assert!(Is::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(true).as_bytes());
    }

    #[test]
    fn test_two_args() {
        let hop = Hop::new();
        let mut args = Vec::new();
        hop.state().key_or_insert_with(b"foo", Value::string);
        args.push(b"foo".to_vec());
        hop.state().key_or_insert_with(b"bar", Value::string);
        args.push(b"bar".to_vec());

        let req = Request::new_with_type(CommandId::Is, Some(args), KeyType::String);
        let mut resp = Vec::new();

        assert!(Is::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(true).as_bytes());
    }

    #[test]
    fn test_two_mismatched() {
        let hop = Hop::new();
        let mut args = Vec::new();
        hop.state().key_or_insert_with(b"foo", Value::string);
        args.push(b"foo".to_vec());
        hop.state().key_or_insert_with(b"bar", Value::integer);
        args.push(b"bar".to_vec());

        let req = Request::new_with_type(CommandId::Is, Some(args), KeyType::String);
        let mut resp = Vec::new();

        assert!(Is::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(false).as_bytes());
    }

    #[test]
    fn test_no_arguments() {
        let hop = Hop::new();

        let req = Request::new_with_type(CommandId::Is, None, KeyType::Bytes);
        let mut resp = Vec::new();

        assert!(matches!(
            Is::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::ArgumentRetrieval)
        ));
    }

    #[test]
    fn test_key_type_unspecified() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());

        let req = Request::new(CommandId::Is, Some(args));
        let mut resp = Vec::new();

        assert!(matches!(
            Is::dispatch(&hop, &req, &mut resp),
            Err(DispatchError::KeyTypeRequired)
        ));
    }
}
