use crate::{
    command::{response, Dispatch, DispatchError, DispatchResult, Request},
    state::{
        object::{Boolean, Bytes, Float, Integer, List, Map, Set as SetObject, Str},
        KeyType, Value,
    },
    Hop,
};
use alloc::{borrow::ToOwned, vec::Vec};

pub struct Set;

impl Set {
    fn boolean(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req.typed_arg(1).ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut boolean = hop
            .state()
            .typed_key::<Boolean>(key)
            .ok_or(DispatchError::WrongType)?;

        *boolean = arg;

        response::write_bool(resp, arg);

        Ok(())
    }

    fn bytes(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req
            .typed_arg::<&[u8]>(1)
            .ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut bytes = hop
            .state()
            .typed_key::<Bytes>(key)
            .ok_or(DispatchError::WrongType)?;

        *bytes = arg.to_vec();

        response::write_bytes(resp, arg);

        Ok(())
    }

    fn float(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req.typed_arg(1).ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut float = hop
            .state()
            .typed_key::<Float>(key)
            .ok_or(DispatchError::WrongType)?;

        *float = arg;

        response::write_float(resp, arg);

        Ok(())
    }

    fn integer(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req.typed_arg(1).ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut int = hop
            .state()
            .typed_key::<Integer>(key)
            .ok_or(DispatchError::WrongType)?;

        *int = arg;

        response::write_int(resp, arg);

        Ok(())
    }

    fn list(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let args = req.args(1..).ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut list = hop
            .state()
            .typed_key::<List>(key)
            .ok_or(DispatchError::WrongType)?;

        *list = args.to_vec();

        response::write_list(resp, args);

        Ok(())
    }

    fn map(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let args = req.typed_args().ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut map = hop
            .state()
            .typed_key::<Map>(key)
            .ok_or(DispatchError::WrongType)?;

        response::write_map(resp, &args);

        *map = args;

        Ok(())
    }

    fn set(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let args = req.typed_args().ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut set = hop
            .state()
            .typed_key::<SetObject>(key)
            .ok_or(DispatchError::WrongType)?;

        response::write_set(resp, &args);

        *set = args;

        Ok(())
    }

    fn string(hop: &Hop, req: &Request, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let arg = req
            .typed_arg::<&str>(1)
            .ok_or(DispatchError::ArgumentRetrieval)?;
        hop.state().remove(key);
        let mut string = hop
            .state()
            .typed_key::<Str>(key)
            .ok_or(DispatchError::WrongType)?;

        *string = arg.to_owned();

        response::write_str(resp, arg);

        Ok(())
    }
}

impl Dispatch for Set {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        // All types require at least one argument, so let's do that check here.
        if req.arg(1).is_none() {
            return Err(DispatchError::ArgumentRetrieval);
        }

        let key_type = req
            .key_type()
            .unwrap_or_else(|| hop.state().key(key, Value::bytes).value().kind());

        match key_type {
            KeyType::Bytes => Self::bytes(hop, req, resp, key),
            KeyType::Boolean => Self::boolean(hop, req, resp, key),
            KeyType::Float => Self::float(hop, req, resp, key),
            KeyType::Integer => Self::integer(hop, req, resp, key),
            KeyType::List => Self::list(hop, req, resp, key),
            KeyType::Map => Self::map(hop, req, resp, key),
            KeyType::Set => Self::set(hop, req, resp, key),
            KeyType::String => Self::string(hop, req, resp, key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Set;
    use crate::{
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::{
            object::{Boolean, Bytes, Float, Integer, List, Map, Set as SetObject, Str},
            KeyType,
        },
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_types_no_arg() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let mut resp = Vec::new();

        let types = [
            KeyType::Boolean,
            KeyType::Bytes,
            KeyType::Float,
            KeyType::Integer,
            KeyType::List,
            KeyType::Map,
            KeyType::Set,
            KeyType::String,
        ];

        for key_type in &types {
            let req = Request::new_with_type(CommandId::Set, Some(args.clone()), *key_type);

            assert_eq!(
                Set::dispatch(&hop, &req, &mut resp).unwrap_err(),
                DispatchError::ArgumentRetrieval,
            );

            resp.clear();
        }
    }

    #[test]
    fn test_bool() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push([1].to_vec());
        let req = Request::new_with_type(CommandId::Set, Some(args), KeyType::Boolean);
        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(true).as_bytes());
        assert!(hop.state().typed_key::<Boolean>(b"foo").as_deref() == Some(&true));
    }

    #[test]
    fn test_bytes() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push(b"bar baz".to_vec());
        let req = Request::new_with_type(CommandId::Set, Some(args), KeyType::Bytes);
        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(b"bar baz".to_vec()).as_bytes());
        assert!(hop.state().typed_key::<Bytes>(b"foo").as_deref() == Some(&b"bar baz".to_vec()));
    }

    #[test]
    fn test_float() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push(2f64.to_be_bytes().to_vec());
        let req = Request::new_with_type(CommandId::Set, Some(args), KeyType::Float);
        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(2f64).as_bytes());
        assert!(hop.state().typed_key::<Float>(b"foo").as_deref() == Some(&2f64));
    }

    #[test]
    fn test_int() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push(2i64.to_be_bytes().to_vec());
        let req = Request::new_with_type(CommandId::Set, Some(args), KeyType::Integer);
        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(2i64).as_bytes());
        assert!(hop.state().typed_key::<Integer>(b"foo").as_deref() == Some(&2));
    }

    #[test]
    fn test_list_three_entries() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push(b"value1".to_vec());
        args.push(b"value2".to_vec());
        args.push(b"value2".to_vec());
        let req = Request::new_with_type(CommandId::Set, Some(args), KeyType::List);
        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert!(
            hop.state()
                .typed_key::<List>(b"foo")
                .as_deref()
                .map(|x| x.len())
                == Some(3)
        );
    }

    #[test]
    fn test_map_two_entries() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push(b"key1".to_vec());
        args.push(b"value1".to_vec());
        args.push(b"key2".to_vec());
        args.push(b"value2".to_vec());
        let req = Request::new_with_type(CommandId::Set, Some(args), KeyType::Map);
        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert!(
            hop.state()
                .typed_key::<Map>(b"foo")
                .as_deref()
                .map(|x| x.len())
                == Some(2)
        );
    }

    #[test]
    fn test_set_two_entries() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push(b"value1".to_vec());
        args.push(b"value2".to_vec());
        let req = Request::new_with_type(CommandId::Set, Some(args), KeyType::Set);
        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert!(
            hop.state()
                .typed_key::<SetObject>(b"foo")
                .as_deref()
                .map(|x| x.len())
                == Some(2)
        );
    }

    #[test]
    fn test_str() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        args.push("bar".as_bytes().to_vec());
        let req = Request::new_with_type(CommandId::Set, Some(args), KeyType::String);
        let mut resp = Vec::new();

        assert!(Set::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from("bar".to_owned()).as_bytes());
        assert!(hop.state().typed_key::<Str>(b"foo").as_deref() == Some(&"bar".to_owned()));
    }
}
