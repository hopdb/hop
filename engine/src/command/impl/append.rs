use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::{
    state::{
        object::{Bytes, List, Str},
        KeyType,
    },
    Hop,
};
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::str;

pub struct Append;

impl Dispatch for Append {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.arg(0).ok_or(DispatchError::KeyRetrieval)?;
        let args = req.arg(1..).ok_or(DispatchError::ArgumentRetrieval)?;

        match req.key_type() {
            Some(KeyType::Bytes) | None => {
                let mut bytes = hop
                    .state()
                    .typed_key::<Bytes>(key)
                    .ok_or(DispatchError::WrongType)?;

                for arg in args {
                    bytes.extend_from_slice(arg);
                }

                response::write_bytes(resp, bytes.as_ref());
            }
            Some(KeyType::List) => {
                let mut list = hop
                    .state()
                    .typed_key::<List>(key)
                    .ok_or(DispatchError::WrongType)?;

                list.append(&mut args.to_owned());

                response::write_list(resp, &list);
            }
            Some(KeyType::String) => {
                let mut string = hop
                    .state()
                    .typed_key::<Str>(key)
                    .ok_or(DispatchError::WrongType)?;

                for arg in args {
                    if let Ok(arg) = str::from_utf8(arg) {
                        string.push_str(arg);
                    }
                }

                response::write_str(resp, &string);
            }
            Some(_) => return Err(DispatchError::WrongType),
        }

        Ok(())
    }
}
