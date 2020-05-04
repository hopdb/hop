use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::{
    state::{
        object::{Bytes, List, Str},
        KeyType, Value,
    },
    Hop,
};
use alloc::vec::Vec;

pub struct Length;

impl Length {
    fn bytes(hop: &Hop, key: &[u8]) -> DispatchResult<Vec<u8>> {
        let bytes = match hop.state().typed_key::<Bytes>(key) {
            Some(bytes) => bytes,
            None => return Err(DispatchError::WrongType),
        };

        Ok(response::write_int(bytes.len() as i64))
    }

    fn list(hop: &Hop, key: &[u8]) -> DispatchResult<Vec<u8>> {
        let list = match hop.state().typed_key::<List>(key) {
            Some(list) => list,
            None => return Err(DispatchError::WrongType),
        };

        Ok(response::write_int(list.len() as i64))
    }

    fn string(hop: &Hop, key: &[u8]) -> DispatchResult<Vec<u8>> {
        let string = match hop.state().typed_key::<Str>(key) {
            Some(string) => string,
            None => return Err(DispatchError::WrongType),
        };

        Ok(response::write_int(string.chars().count() as i64))
    }
}

impl Dispatch for Length {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Vec<u8>> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        match req.key_type() {
            Some(KeyType::Bytes) => Self::bytes(hop, key),
            Some(KeyType::List) => Self::list(hop, key),
            Some(KeyType::String) => Self::string(hop, key),
            Some(_) => Err(DispatchError::WrongType),
            None => {
                let kind = hop.state().key(key, Value::bytes).value().kind();

                match kind {
                    KeyType::Bytes => Self::bytes(hop, key),
                    KeyType::List => Self::list(hop, key),
                    KeyType::String => Self::string(hop, key),
                    _ => Err(DispatchError::WrongType),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Length;
    use crate::{
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::{borrow::ToOwned, vec::Vec};

    #[test]
    fn test_no_args() {
        let hop = Hop::new();
        let req = Request::new(CommandId::Length, None);

        assert_eq!(
            Length::dispatch(&hop, &req).unwrap_err(),
            DispatchError::KeyRetrieval
        );
    }

    #[test]
    fn test_wrong_type() {
        let hop = Hop::new();

        let mut args = Vec::new();
        args.push(b"foo".to_vec());

        let types = [
            KeyType::Boolean,
            KeyType::Float,
            KeyType::Integer,
            KeyType::Map,
            KeyType::Set,
        ];

        for kind in types.iter() {
            let req = Request::new_with_type(CommandId::Length, Some(args.clone()), *kind);

            assert_eq!(
                Length::dispatch(&hop, &req).unwrap_err(),
                DispatchError::WrongType
            );
        }
    }

    #[test]
    fn test_default_when_key_nonexistent() {
        let hop = Hop::new();
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let req = Request::new(CommandId::Length, Some(args));

        assert_eq!(
            Length::dispatch(&hop, &req).unwrap(),
            Response::from(0).into_bytes()
        );
        assert_eq!(hop.state().0.len(), 1);
    }

    #[test]
    fn test_default_when_bytes_exists() {
        let hop = Hop::new();
        hop.state()
            .0
            .insert(b"foo".to_vec(), Value::Bytes([1, 2, 3].to_vec()));
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let req = Request::new(CommandId::Length, Some(args));

        assert_eq!(
            Length::dispatch(&hop, &req).unwrap(),
            Response::from(3).into_bytes()
        );
    }

    #[test]
    fn test_default_when_list_exists() {
        let hop = Hop::new();
        let mut list = Vec::new();
        list.push(b"db".to_vec());

        hop.state().0.insert(b"hop".to_vec(), Value::List(list));
        let mut args = Vec::new();
        args.push(b"hop".to_vec());
        let req = Request::new(CommandId::Length, Some(args));

        assert_eq!(
            Length::dispatch(&hop, &req).unwrap(),
            Response::from(1).into_bytes()
        );
    }

    #[test]
    fn test_default_when_string_exists() {
        let hop = Hop::new();
        hop.state()
            .0
            .insert(b"foo".to_vec(), Value::String("1234".to_owned()));

        let cowboy = "🤠";
        assert_eq!(cowboy.len(), 4);
        hop.state()
            .0
            .insert(b"cowboy".to_vec(), Value::String(cowboy.to_owned()));

        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let req = Request::new(CommandId::Length, Some(args.clone()));

        // length of a simple string, 4 bytes and 4 chars
        assert_eq!(
            Length::dispatch(&hop, &req).unwrap(),
            Response::from(4).into_bytes()
        );

        // length of a simple string, 4 bytes but 1 char
        args.pop();
        args.push(b"cowboy".to_vec());
        let req = Request::new(CommandId::Length, Some(args.clone()));
        assert_eq!(
            Length::dispatch(&hop, &req).unwrap(),
            Response::from(1).into_bytes()
        );
    }
}
