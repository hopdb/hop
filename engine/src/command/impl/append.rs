use super::super::{
    request::Arguments, response, Dispatch, DispatchError, DispatchResult, Request,
};
use crate::{
    state::{KeyType, Value},
    Hop,
};
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::str;

pub struct Append;

impl Append {
    fn bytes(hop: &Hop, args: Arguments<'_>, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let mut key = hop.state().key_or_insert_with(key, Value::bytes);
        let bytes = key.as_bytes_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        for arg in args {
            bytes.extend_from_slice(arg);
        }

        response::write_bytes(resp, bytes.as_ref());

        Ok(())
    }

    fn list(hop: &Hop, args: Arguments<'_>, resp: &mut Vec<u8>, key: &[u8]) -> DispatchResult<()> {
        let mut key = hop.state().key_or_insert_with(key, Value::list);
        let list = key.as_list_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        list.append(&mut args.map(ToOwned::to_owned).collect());

        response::write_list(resp, list.iter());

        Ok(())
    }

    fn string(
        hop: &Hop,
        args: Arguments<'_>,
        resp: &mut Vec<u8>,
        key: &[u8],
    ) -> DispatchResult<()> {
        let mut key = hop.state().key_or_insert_with(key, Value::string);
        let string = key.as_string_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        for arg in args {
            if let Ok(arg) = str::from_utf8(arg) {
                string.push_str(arg);
            }
        }

        response::write_str(resp, &string);

        Ok(())
    }
}

impl Dispatch for Append {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.arg(0).ok_or(DispatchError::KeyUnspecified)?;
        let args = req.args(1..).ok_or(DispatchError::ArgumentRetrieval)?;
        let key_type = req
            .key_type()
            .or_else(|| hop.state().key_type(key))
            .unwrap_or(KeyType::Bytes);

        match key_type {
            KeyType::Bytes => Self::bytes(hop, args, resp, key),
            KeyType::List => Self::list(hop, args, resp, key),
            KeyType::String => Self::string(hop, args, resp, key),
            _ => Err(DispatchError::KeyTypeDifferent),
        }
    }
}
