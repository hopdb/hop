mod context;

pub use context::{Context, Instruction, ParseError};

use super::{request::ParseError as RequestParseError, DispatchError};
use crate::state::Value;
use alloc::{string::String, vec::Vec};
use core::convert::TryFrom;
use dashmap::{DashMap, DashSet};

/// The type of response value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ResponseType {
    Boolean = 0,
    Bytes = 1,
    Float = 2,
    Integer = 3,
    List = 4,
    Map = 5,
    Set = 6,
    String = 7,
    ParseError = 8,
    DispatchError = 9,
}

impl TryFrom<u8> for ResponseType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Boolean,
            1 => Self::Bytes,
            2 => Self::Float,
            3 => Self::Integer,
            4 => Self::List,
            5 => Self::Map,
            6 => Self::Set,
            7 => Self::String,
            8 => Self::ParseError,
            9 => Self::DispatchError,
            _ => return Err(()),
        })
    }
}

#[derive(Debug)]
pub enum Response {
    DispatchError(DispatchError),
    ParseError(RequestParseError),
    Value(Value),
}

impl Response {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            Self::DispatchError(err) => write_dispatch_error(&mut buf, *err),
            Self::ParseError(err) => write_parse_error(&mut buf, *err),
            Self::Value(Value::Boolean(boolean)) => write_bool(&mut buf, *boolean),
            Self::Value(Value::Bytes(bytes)) => write_bytes(&mut buf, bytes),
            Self::Value(Value::Float(float)) => write_float(&mut buf, *float),
            Self::Value(Value::Integer(int)) => write_int(&mut buf, *int),
            Self::Value(Value::List(list)) => write_list(&mut buf, list),
            Self::Value(Value::Map(map)) => write_map(&mut buf, map),
            Self::Value(Value::Set(set)) => write_set(&mut buf, set),
            Self::Value(Value::String(string)) => write_str(&mut buf, string),
        }

        buf
    }
}

impl From<bool> for Response {
    fn from(value: bool) -> Self {
        Self::Value(Value::Boolean(value))
    }
}

impl From<Vec<u8>> for Response {
    fn from(value: Vec<u8>) -> Self {
        Self::Value(Value::Bytes(value))
    }
}

impl From<f64> for Response {
    fn from(value: f64) -> Self {
        Self::Value(Value::Float(value))
    }
}

impl From<i64> for Response {
    fn from(value: i64) -> Self {
        Self::Value(Value::Integer(value))
    }
}

impl From<Vec<Vec<u8>>> for Response {
    fn from(value: Vec<Vec<u8>>) -> Self {
        Self::Value(Value::List(value))
    }
}

impl From<DashMap<Vec<u8>, Vec<u8>>> for Response {
    fn from(value: DashMap<Vec<u8>, Vec<u8>>) -> Self {
        Self::Value(Value::Map(value))
    }
}

impl From<DashSet<Vec<u8>>> for Response {
    fn from(value: DashSet<Vec<u8>>) -> Self {
        Self::Value(Value::Set(value))
    }
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        Self::Value(Value::String(value))
    }
}

impl From<DispatchError> for Response {
    fn from(value: DispatchError) -> Self {
        Self::DispatchError(value)
    }
}

impl From<RequestParseError> for Response {
    fn from(value: RequestParseError) -> Self {
        Self::ParseError(value)
    }
}

pub fn write_bool(to: &mut Vec<u8>, value: bool) {
    to.push(ResponseType::Boolean as u8);
    to.push(if value { 1 } else { 0 });
}

pub fn write_bytes(to: &mut Vec<u8>, value: &[u8]) {
    let len = value.len() as u32;

    to.push(ResponseType::Bytes as u8);
    to.extend_from_slice(&len.to_be_bytes());
    to.extend_from_slice(value);
}

pub fn write_dispatch_error(to: &mut Vec<u8>, value: DispatchError) {
    to.push(ResponseType::DispatchError as u8);
    to.push(value as u8);
}

pub fn write_parse_error(to: &mut Vec<u8>, value: RequestParseError) {
    to.push(ResponseType::ParseError as u8);
    to.push(value as u8);
}

pub fn write_float(to: &mut Vec<u8>, value: f64) {
    to.push(ResponseType::Float as u8);
    to.extend_from_slice(&value.to_be_bytes());
}

pub fn write_int(to: &mut Vec<u8>, value: i64) {
    to.push(ResponseType::Integer as u8);
    to.extend_from_slice(&value.to_be_bytes());
}

pub fn write_list(to: &mut Vec<u8>, value: &[Vec<u8>]) {
    to.push(ResponseType::List as u8);

    // The length of the list.
    to.extend_from_slice(&(value.len() as u16).to_be_bytes());

    // Now for each list item, push its length and then the item itself.
    for item in value {
        let len = item.len() as u32;

        to.extend_from_slice(&len.to_be_bytes());
        to.extend_from_slice(item);
    }
}

pub fn write_map(to: &mut Vec<u8>, value: &DashMap<Vec<u8>, Vec<u8>>) {
    to.push(ResponseType::Map as u8);

    // Maps can only contain up to u16 items.
    to.extend_from_slice(&(value.len() as u16).to_be_bytes());

    for item in value.iter() {
        let (key, value) = item.pair();

        let key_len = key.len() as u8;
        let value_len = value.len() as u32;

        to.push(key_len);
        to.extend_from_slice(key);
        to.extend_from_slice(&value_len.to_be_bytes());
        to.extend_from_slice(value);
    }
}

pub fn write_set(to: &mut Vec<u8>, value: &DashSet<Vec<u8>>) {
    to.push(ResponseType::Set as u8);

    // Sets can only contain up to u16 items.
    to.extend_from_slice(&(value.len() as u16).to_be_bytes());

    for item in value.iter() {
        let len = item.len() as u16;

        to.extend_from_slice(&len.to_be_bytes());
        to.extend_from_slice(item.key());
    }
}

pub fn write_str(to: &mut Vec<u8>, value: &str) {
    let len = value.len() as u32;

    to.push(ResponseType::String as u8);

    to.extend_from_slice(&len.to_be_bytes());
    to.extend_from_slice(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::{Response, ResponseType};
    use alloc::{borrow::ToOwned, string::String, vec::Vec};
    use dashmap::{DashMap, DashSet};

    #[test]
    fn test_bool() {
        assert_eq!(
            Response::from(false).as_bytes(),
            [ResponseType::Boolean as u8, 0]
        );
        assert_eq!(
            Response::from(true).as_bytes(),
            [ResponseType::Boolean as u8, 1]
        );
    }

    #[test]
    fn test_bytes() {
        assert_eq!(
            Response::from(b"hopdb".to_vec()).as_bytes(),
            [
                ResponseType::Bytes as u8,
                // length, max u32
                0,
                0,
                0,
                5,
                b'h',
                b'o',
                b'p',
                b'd',
                b'b',
            ],
        );
    }

    #[test]
    fn test_bytes_empty() {
        assert_eq!(
            Response::from(b"".to_vec()).as_bytes(),
            [
                ResponseType::Bytes as u8,
                // length, max u32
                0,
                0,
                0,
                0,
            ],
        );
    }

    #[test]
    fn test_float() {
        assert_eq!(
            Response::from(7.4).as_bytes(),
            [
                ResponseType::Float as u8,
                64,
                29,
                153,
                153,
                153,
                153,
                153,
                154
            ],
        );
    }

    #[test]
    fn test_int() {
        assert_eq!(
            Response::from(7).as_bytes(),
            [ResponseType::Integer as u8, 0, 0, 0, 0, 0, 0, 0, 7],
        );
        assert_eq!(
            Response::from(-7).as_bytes(),
            [
                ResponseType::Integer as u8,
                255,
                255,
                255,
                255,
                255,
                255,
                255,
                249
            ],
        );
        assert_eq!(
            Response::from(68125).as_bytes(),
            [ResponseType::Integer as u8, 0, 0, 0, 0, 0, 1, 10, 29],
        );
    }

    #[test]
    fn test_list() {
        let mut list = Vec::new();
        list.push(b"hop".to_vec());
        list.push(b"db".to_vec());

        assert_eq!(
            Response::from(list).as_bytes(),
            [
                ResponseType::List as u8,
                // length of the list
                0,
                2,
                // length of first item ("hop")
                0,
                0,
                0,
                3,
                // first item ("hop")
                b'h',
                b'o',
                b'p',
                // length of second item ("db")
                0,
                0,
                0,
                2,
                // second item ("db")
                b'd',
                b'b',
            ],
        );
    }

    #[test]
    fn test_list_empty() {
        let v: Vec<Vec<_>> = Vec::new();

        assert_eq!(
            Response::from(v).as_bytes(),
            [
                ResponseType::List as u8,
                // length of the list
                0,
                0,
            ],
        );
    }

    #[test]
    fn test_map() {
        let map = DashMap::new();
        map.insert(b"f".to_vec(), b"foo".to_vec());
        map.insert(b"123".to_vec(), Vec::new());

        // Ordering can be random, so we need to check if it's one of either of
        // these.
        let possible_values = [
            [
                ResponseType::Map as u8,
                // length of map (there can be up to u16 items)
                0,
                2,
                // length of first key, u8 ("123")
                3,
                // first key ("123")
                b'1',
                b'2',
                b'3',
                // length of first value, u32 (nothing)
                0,
                0,
                0,
                0,
                // first value (nothing)
                // length of second key, u8 ("f")
                1,
                // second key ("f")
                b'f',
                // length of second value, u32 ("foo")
                0,
                0,
                0,
                3,
                // second value ("foo")
                b'f',
                b'o',
                b'o',
            ],
            [
                ResponseType::Map as u8,
                // length of map (there can be up to u16 items)
                0,
                2,
                // length of first key, u8 ("f")
                1,
                // first key ("f")
                b'f',
                // length of first value, u32 ("foo")
                0,
                0,
                0,
                3,
                // first value ("foo")
                b'f',
                b'o',
                b'o',
                // length of second key, u8 ("123")
                3,
                // second key ("123")
                b'1',
                b'2',
                b'3',
                // length of second value, u32 (nothing)
                0,
                0,
                0,
                0,
                // second value (nothing)
            ],
        ];

        let resp = Response::from(map).as_bytes();

        assert!(possible_values.iter().any(|v| v == resp.as_slice()));
    }

    #[test]
    fn test_map_empty() {
        assert_eq!(
            Response::from(DashMap::new()).as_bytes(),
            [ResponseType::Map as u8, 0, 0]
        );
    }

    #[test]
    fn test_set() {
        let map = DashSet::new();
        map.insert(b"hop".to_vec());
        map.insert(b"db".to_vec());

        // Ordering can be random, so we need to check if it's one of either of
        // these.
        let possible_values = [
            [
                ResponseType::Set as u8,
                // length of set (there can be up to u16 items)
                0,
                2,
                // length of first item, u16 ("hop")
                0,
                3,
                // first item ("hop")
                b'h',
                b'o',
                b'p',
                // length of second item, u16 ("db")
                0,
                2,
                // second item ("db")
                b'd',
                b'b',
            ],
            [
                ResponseType::Set as u8,
                // length of set (there can be up to u16 items)
                0,
                2,
                // length of first item, u16 ("hop")
                0,
                2,
                // first item ("hop")
                b'd',
                b'b',
                // length of second item, u16 ("db")
                0,
                3,
                // second item ("db")
                b'h',
                b'o',
                b'p',
            ],
        ];

        let resp = Response::from(map).as_bytes();

        assert!(possible_values.iter().any(|v| v == resp.as_slice()));
    }

    #[test]
    fn test_set_empty() {
        assert_eq!(
            Response::from(DashSet::new()).as_bytes(),
            [ResponseType::Set as u8, 0, 0]
        );
    }

    #[test]
    fn test_str() {
        assert_eq!(
            Response::from("foo bar baz".to_owned()).as_bytes(),
            [
                ResponseType::String as u8,
                // 4 bytes string length
                0,
                0,
                0,
                11,
                // 11 characters
                b'f',
                b'o',
                b'o',
                b' ',
                b'b',
                b'a',
                b'r',
                b' ',
                b'b',
                b'a',
                b'z',
            ],
        );
    }

    #[test]
    fn test_str_empty() {
        assert_eq!(
            Response::from(String::new()).as_bytes(),
            [ResponseType::String as u8, 0, 0, 0, 0]
        );
    }
}
