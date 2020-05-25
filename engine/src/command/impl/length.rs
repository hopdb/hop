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
    fn bytes(hop: &Hop, key: &[u8], resp: &mut Vec<u8>) -> DispatchResult<()> {
        let bytes = match hop.state().typed_key::<Bytes>(key) {
            Some(bytes) => bytes,
            None => return Err(DispatchError::KeyTypeDifferent),
        };

        response::write_int(resp, bytes.len() as i64);

        Ok(())
    }

    fn list(hop: &Hop, key: &[u8], resp: &mut Vec<u8>) -> DispatchResult<()> {
        let list = match hop.state().typed_key::<List>(key) {
            Some(list) => list,
            None => return Err(DispatchError::KeyTypeDifferent),
        };

        response::write_int(resp, list.len() as i64);

        Ok(())
    }

    fn string(hop: &Hop, key: &[u8], resp: &mut Vec<u8>) -> DispatchResult<()> {
        let string = match hop.state().typed_key::<Str>(key) {
            Some(string) => string,
            None => return Err(DispatchError::KeyTypeDifferent),
        };

        response::write_int(resp, string.chars().count() as i64);

        Ok(())
    }
}

impl Dispatch for Length {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        match req.key_type() {
            Some(KeyType::Bytes) => Self::bytes(hop, key, resp),
            Some(KeyType::List) => Self::list(hop, key, resp),
            Some(KeyType::String) => Self::string(hop, key, resp),
            Some(_) => Err(DispatchError::KeyTypeInvalid),
            None => {
                let kind = hop
                    .state()
                    .key_or_insert_with(key, Value::bytes)
                    .value()
                    .kind();

                match kind {
                    KeyType::Bytes => Self::bytes(hop, key, resp),
                    KeyType::List => Self::list(hop, key, resp),
                    KeyType::String => Self::string(hop, key, resp),
                    _ => Err(DispatchError::KeyTypeInvalid),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Length;
    use crate::{
        command::{request::RequestBuilder, CommandId, Dispatch, DispatchError, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::{borrow::ToOwned, vec::Vec};

    #[test]
    fn test_no_args() {
        let req = RequestBuilder::new(CommandId::Length).into_request();

        let mut resp = Vec::new();
        let hop = Hop::new();

        assert_eq!(
            Length::dispatch(&hop, &req, &mut resp).unwrap_err(),
            DispatchError::KeyUnspecified
        );
    }

    #[test]
    fn test_invalid_key_type() {
        let mut builder = RequestBuilder::new(CommandId::Length);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());

        let types = [
            KeyType::Boolean,
            KeyType::Float,
            KeyType::Integer,
            KeyType::Map,
            KeyType::Set,
        ];

        let mut resp = Vec::new();
        let hop = Hop::new();

        for kind in types.iter() {
            let mut builder = builder.clone();
            builder.key_type(*kind);
            let req = builder.into_request();

            assert_eq!(
                Length::dispatch(&hop, &req, &mut resp).unwrap_err(),
                DispatchError::KeyTypeInvalid
            );

            resp.clear();
        }
    }

    #[test]
    fn test_default_when_key_nonexistent() {
        let mut builder = RequestBuilder::new(CommandId::Length);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();
        let mut resp = Vec::new();

        assert!(Length::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(0).as_bytes());
        assert_eq!(hop.state().0.len(), 1);
    }

    #[test]
    fn test_default_when_bytes_exists() {
        let mut builder = RequestBuilder::new(CommandId::Length);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let mut resp = Vec::new();
        let hop = Hop::new();
        hop.state()
            .0
            .insert(b"foo".to_vec(), Value::Bytes([1, 2, 3].to_vec()));

        assert!(Length::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(3).as_bytes());
    }

    #[test]
    fn test_default_when_list_exists() {
        let mut builder = RequestBuilder::new(CommandId::Length);
        assert!(builder.bytes(b"hop".as_ref()).is_ok());
        let req = builder.into_request();

        let mut resp = Vec::new();
        let hop = Hop::new();
        let mut list = Vec::new();
        list.push(b"db".to_vec());
        hop.state().0.insert(b"hop".to_vec(), Value::List(list));

        assert!(Length::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(1).as_bytes());
    }

    #[test]
    fn test_default_when_string_exists() {
        let mut builder = RequestBuilder::new(CommandId::Length);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();
        hop.state()
            .0
            .insert(b"foo".to_vec(), Value::String("1234".to_owned()));

        let cowboy = "🤠";
        assert_eq!(cowboy.len(), 4);
        hop.state()
            .0
            .insert(b"cowboy".to_vec(), Value::String(cowboy.to_owned()));

        let mut builder = RequestBuilder::new(CommandId::Length);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());

        let mut resp = Vec::new();

        // length of a simple string, 4 bytes and 4 chars
        assert!(Length::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(4).as_bytes());

        resp.clear();

        // length of a simple string, 4 bytes but 1 char
        let mut builder = RequestBuilder::new(CommandId::Length);
        assert!(builder.bytes(b"cowboy".as_ref()).is_ok());
        let req = builder.into_request();

        assert!(Length::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(1).as_bytes());
    }
}
