use super::prelude::*;
use crate::state::{
    object::{Bytes, List, Str},
    KeyType,
};
use alloc::borrow::ToOwned;
use core::str;

pub struct Append;

impl Dispatch for Append {
    fn dispatch(hop: &Hop, req: &mut Request) -> Result<Response> {
        let key = req.arg(0).ok_or(Error::KeyRetrieval)?;
        let args = req.arg(1..).ok_or(Error::ArgumentRetrieval)?;

        match req.key_type() {
            Some(KeyType::Bytes) | None => {
                let mut bytes = hop
                    .state()
                    .typed_key::<Bytes>(key)
                    .ok_or(Error::WrongType)?;

                for arg in args {
                    bytes.extend_from_slice(arg);
                }

                Ok(Response::from(bytes.as_slice()))
            }
            Some(KeyType::List) => {
                let mut list = hop.state().typed_key::<List>(key).ok_or(Error::WrongType)?;

                list.append(&mut args.to_owned());

                Ok(Response::from(list.as_slice()))
            }
            Some(KeyType::String) => {
                let mut string = hop.state().typed_key::<Str>(key).ok_or(Error::WrongType)?;

                for arg in args {
                    if let Ok(arg) = str::from_utf8(arg) {
                        string.push_str(arg);
                    }
                }

                Ok(Response::from(string.as_str()))
            }
            Some(_) => Err(Error::WrongType),
        }
    }
}
