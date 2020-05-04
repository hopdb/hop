use super::{request::ParseError, DispatchError};
use crate::state::Value;
use alloc::{string::String, vec::Vec};
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

#[derive(Debug)]
pub enum Response {
    DispatchError(DispatchError),
    ParseError(ParseError),
    Value(Value),
}

impl Response {
    pub fn into_bytes(&self) -> Vec<u8> {
        match self {
            Self::DispatchError(err) => write_dispatch_error(*err),
            Self::ParseError(err) => write_parse_error(*err),
            Self::Value(Value::Boolean(boolean)) => write_bool(*boolean),
            Self::Value(Value::Bytes(bytes)) => write_bytes(bytes),
            Self::Value(Value::Float(float)) => write_float(*float),
            Self::Value(Value::Integer(int)) => write_int(*int),
            Self::Value(Value::List(list)) => write_list(list),
            Self::Value(Value::Map(map)) => write_map(map),
            Self::Value(Value::Set(set)) => write_set(set),
            Self::Value(Value::String(string)) => write_str(string),
        }
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

pub fn write_bool(value: bool) -> Vec<u8> {
    let mut buf = Vec::with_capacity(2);

    buf.push(ResponseType::Boolean as u8);
    buf.push(if value { 1 } else { 0 });

    buf
}

pub fn write_bytes(value: &[u8]) -> Vec<u8> {
    let len = value.len() as u32;

    let mut buf = Vec::with_capacity(5 + len as usize);

    buf.push(ResponseType::Bytes as u8);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(value);

    buf
}

pub fn write_dispatch_error(value: DispatchError) -> Vec<u8> {
    let mut buf = Vec::with_capacity(2);

    buf.push(ResponseType::DispatchError as u8);
    buf.push(value as u8);

    buf
}

pub fn write_parse_error(value: ParseError) -> Vec<u8> {
    let mut buf = Vec::with_capacity(2);

    buf.push(ResponseType::ParseError as u8);
    buf.push(value as u8);

    buf
}

pub fn write_float(value: f64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(9);

    buf.push(ResponseType::Float as u8);
    buf.extend_from_slice(&value.to_be_bytes());

    buf
}

pub fn write_int(value: i64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(9);

    buf.push(ResponseType::Integer as u8);
    buf.extend_from_slice(&value.to_be_bytes());

    buf
}

pub fn write_list(value: &[Vec<u8>]) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.push(ResponseType::List as u8);

    // The length of the list.
    buf.extend_from_slice(&(value.len() as u16).to_be_bytes());

    // Now for each list item, push its length and then the item itself.
    for item in value {
        let len = item.len() as u32;

        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(item);
    }

    buf
}

pub fn write_map(value: &DashMap<Vec<u8>, Vec<u8>>) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.push(ResponseType::Map as u8);

    // Maps can only contain up to u16 items.
    buf.extend_from_slice(&(value.len() as u16).to_be_bytes());

    for item in value.iter() {
        let (key, value) = item.pair();

        let key_len = key.len() as u8;
        let value_len = value.len() as u32;

        buf.push(key_len);
        buf.extend_from_slice(key);
        buf.extend_from_slice(&value_len.to_be_bytes());
        buf.extend_from_slice(value);
    }

    buf
}

pub fn write_set(value: &DashSet<Vec<u8>>) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.push(ResponseType::Set as u8);

    // Sets can only contain up to u16 items.
    buf.extend_from_slice(&(value.len() as u16).to_be_bytes());

    for item in value.iter() {
        let len = item.len() as u16;

        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(item.key());
    }

    buf
}

pub fn write_str(value: &str) -> Vec<u8> {
    let len = value.len() as u32;

    let mut buf = Vec::with_capacity(5 + len as usize);
    buf.push(ResponseType::String as u8);

    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(value.as_bytes());

    buf
}

#[cfg(test)]
mod tests {
    use super::{Response, ResponseType};
    use alloc::{borrow::ToOwned, string::String, vec::Vec};
    use dashmap::{DashMap, DashSet};

    #[test]
    fn test_bool() {
        assert_eq!(
            Response::from(false).into_bytes(),
            [ResponseType::Boolean as u8, 0]
        );
        assert_eq!(
            Response::from(true).into_bytes(),
            [ResponseType::Boolean as u8, 1]
        );
    }

    #[test]
    fn test_bytes() {
        assert_eq!(
            Response::from(b"hopdb".to_vec()).into_bytes(),
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
            Response::from(b"".to_vec()).into_bytes(),
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
            Response::from(7.4).into_bytes(),
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
            Response::from(7).into_bytes(),
            [ResponseType::Integer as u8, 0, 0, 0, 0, 0, 0, 0, 7],
        );
        assert_eq!(
            Response::from(-7).into_bytes(),
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
            Response::from(68125).into_bytes(),
            [ResponseType::Integer as u8, 0, 0, 0, 0, 0, 1, 10, 29],
        );
    }

    #[test]
    fn test_list() {
        let mut list = Vec::new();
        list.push(b"hop".to_vec());
        list.push(b"db".to_vec());

        assert_eq!(
            Response::from(list).into_bytes(),
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
            Response::from(v).into_bytes(),
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

        let resp = Response::from(map).into_bytes();

        assert!(possible_values.iter().any(|v| v == resp.as_slice()));
    }

    #[test]
    fn test_map_empty() {
        assert_eq!(
            Response::from(DashMap::new()).into_bytes(),
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

        let resp = Response::from(map).into_bytes();

        assert!(possible_values.iter().any(|v| v == resp.as_slice()));
    }

    #[test]
    fn test_set_empty() {
        assert_eq!(
            Response::from(DashSet::new()).into_bytes(),
            [ResponseType::Set as u8, 0, 0]
        );
    }

    #[test]
    fn test_str() {
        assert_eq!(
            Response::from("foo bar baz".to_owned()).into_bytes(),
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
            Response::from(String::new()).into_bytes(),
            [ResponseType::String as u8, 0, 0, 0, 0]
        );
    }
}
