use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::{
    state::{KeyType, Value},
    Hop,
};
use alloc::vec::Vec;

pub struct Length;

impl Length {
    fn bytes(hop: &Hop, key: &[u8], resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = hop
            .state()
            .key_ref(key)
            .ok_or(DispatchError::KeyNonexistent)?;
        let bytes = key.as_bytes_ref().ok_or(DispatchError::KeyTypeDifferent)?;

        response::write_int(resp, bytes.len() as i64);

        Ok(())
    }

    fn list(hop: &Hop, key: &[u8], resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = hop
            .state()
            .key_ref(key)
            .ok_or(DispatchError::KeyNonexistent)?;
        let list = key.as_list_ref().ok_or(DispatchError::KeyTypeDifferent)?;

        response::write_int(resp, list.len() as i64);

        Ok(())
    }

    fn string(hop: &Hop, key: &[u8], resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = hop
            .state()
            .key_ref(key)
            .ok_or(DispatchError::KeyNonexistent)?;
        let string = key.as_string_ref().ok_or(DispatchError::KeyTypeDifferent)?;

        response::write_int(resp, string.chars().count() as i64);

        Ok(())
    }
}

impl Dispatch for Length {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;
        let key_type = req.key_type().unwrap_or_else(|| {
            hop.state()
                .key_or_insert_with(key, Value::bytes)
                .value()
                .kind()
        });

        match key_type {
            KeyType::Bytes => Self::bytes(hop, key, resp),
            KeyType::List => Self::list(hop, key, resp),
            KeyType::String => Self::string(hop, key, resp),
            _ => Err(DispatchError::KeyTypeInvalid),
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
            let mut builder = RequestBuilder::new_with_key_type(CommandId::Length, *kind);
            assert!(builder.bytes(b"foo".as_ref()).is_ok());
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
