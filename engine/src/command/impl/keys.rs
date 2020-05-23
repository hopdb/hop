use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::{
    state::{object::Map, KeyType},
    Hop,
};
use alloc::vec::Vec;

pub struct Keys;

impl Keys {
    fn map(hop: &Hop, key: &[u8], resp: &mut Vec<u8>) -> DispatchResult<()> {
        let map = hop
            .state()
            .typed_key::<Map>(key)
            .ok_or_else(|| DispatchError::KeyTypeDifferent)?;
        let iter = map.iter().map(|r| r.key().to_vec());

        response::write_list(resp, iter);

        Ok(())
    }
}

impl Dispatch for Keys {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        match req.key_type() {
            Some(KeyType::Map) => Self::map(hop, key, resp),
            Some(_) => Err(DispatchError::KeyTypeInvalid),
            None => Self::map(hop, key, resp),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Keys;
    use crate::{
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::{KeyType, Value},
        Hop,
    };
    use alloc::vec::Vec;
    use dashmap::DashMap;

    #[test]
    fn test_map_no_key_type_two_pairs() {
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let hop = Hop::new();
        let req = Request::new(CommandId::Keys, Some(args));

        let mut resp = Vec::new();

        let map = DashMap::new();
        map.insert(b"key1".to_vec(), b"value2".to_vec());
        map.insert(b"key2".to_vec(), b"value2".to_vec());
        hop.state().insert(b"foo".to_vec(), Value::Map(map));

        assert!(Keys::dispatch(&hop, &req, &mut resp).is_ok());
        let expected1 = Response::from([b"key1".to_vec(), b"key2".to_vec()].to_vec()).as_bytes();
        let expected2 = Response::from([b"key2".to_vec(), b"key1".to_vec()].to_vec()).as_bytes();
        assert!(resp == expected1 || resp == expected2);
    }

    #[test]
    fn test_map_key_type() {
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let hop = Hop::new();
        let req = Request::new_with_type(CommandId::Keys, Some(args), KeyType::Map);

        let mut resp = Vec::new();

        let map = DashMap::new();
        map.insert(b"key".to_vec(), b"value".to_vec());
        hop.state().insert(b"foo".to_vec(), Value::Map(map));

        assert!(Keys::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from([b"key".to_vec()].to_vec()).as_bytes());
    }

    #[test]
    fn test_key_type_invalid() {
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let hop = Hop::new();
        let req = Request::new_with_type(CommandId::Keys, Some(args), KeyType::Integer);

        let mut resp = Vec::new();

        assert_eq!(
            DispatchError::KeyTypeInvalid,
            Keys::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }

    #[test]
    fn test_key_type_different() {
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let hop = Hop::new();
        let req = Request::new_with_type(CommandId::Keys, Some(args), KeyType::Map);

        let mut resp = Vec::new();

        hop.state().insert(b"foo".to_vec(), Value::Integer(1));
        assert_eq!(
            DispatchError::KeyTypeDifferent,
            Keys::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }
}
