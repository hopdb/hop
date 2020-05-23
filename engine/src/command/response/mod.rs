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
        self.copy_to(&mut buf);

        buf
    }

    pub fn copy_to(&self, buf: &mut Vec<u8>) {
        match self {
            Self::DispatchError(err) => write_dispatch_error(buf, *err),
            Self::ParseError(err) => write_parse_error(buf, *err),
            Self::Value(value) => write_value(buf, value),
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
    // kind + 1 byte bool
    to.extend_from_slice(&2u32.to_be_bytes());
    to.push(ResponseType::Boolean as u8);
    to.push(if value { 1 } else { 0 });
}

pub fn write_bytes(to: &mut Vec<u8>, value: &[u8]) {
    let len = value.len() as u32;

    // kind + 4 byte bytestream len + value len
    let response_len = 1 + 4 + len;
    to.extend_from_slice(&response_len.to_be_bytes());
    to.push(ResponseType::Bytes as u8);
    to.extend_from_slice(&len.to_be_bytes());
    to.extend_from_slice(value);
}

pub fn write_dispatch_error(to: &mut Vec<u8>, value: DispatchError) {
    // kind + 1 byte error
    to.extend_from_slice(&2u32.to_be_bytes());
    to.push(ResponseType::DispatchError as u8);
    to.push(value as u8);
}

pub fn write_parse_error(to: &mut Vec<u8>, value: RequestParseError) {
    // kind + 1 byte error
    to.extend_from_slice(&2u32.to_be_bytes());
    to.push(ResponseType::ParseError as u8);
    to.push(value as u8);
}

pub fn write_float(to: &mut Vec<u8>, value: f64) {
    // kind + 8 byte int
    to.extend_from_slice(&9u32.to_be_bytes());
    to.push(ResponseType::Float as u8);
    to.extend_from_slice(&value.to_be_bytes());
}

pub fn write_int(to: &mut Vec<u8>, value: i64) {
    // kind + 8 byte int
    to.extend_from_slice(&9u32.to_be_bytes());
    to.push(ResponseType::Integer as u8);
    to.extend_from_slice(&value.to_be_bytes());
}

pub fn write_list<T: IntoIterator<Item = U>, U: AsRef<[u8]>>(to: &mut Vec<u8>, value: T) {
    // We're going to keep a note of how long the buffer to write to is now and
    // pre-insert 4 bytes set to 0.
    //
    // During iteration, a total length of the message while it's being written
    // to the buffer will be counted and tracked. When it's done, we'll write
    // over these 4 bytes with the actual counted length.
    //
    // This needs to be done because core iterators can't efficiently be
    // "rewinded" back to the beginning, so you'd have to clone the entire
    // target of the iterator to iterate over it twice.
    let start = to.len();
    to.extend_from_slice(&[0, 0, 0, 0]);

    to.push(ResponseType::List as u8);

    // 1 byte for response type + 2 bytes for item length
    let mut msg_len = 1 + 2;

    // Same story here, we'll pre-write 2 0-value bytes for the list length.
    to.push(0);
    to.push(0);

    let mut item_count = 0u16;

    // Now for each list item, push its length and then the item itself.
    for item in value {
        item_count += 1;

        let item = item.as_ref();

        let item_len = item.len() as u32;
        // item len in bytes + item len
        msg_len += 4 + item_len;

        to.extend_from_slice(&item_len.to_be_bytes());
        to.extend_from_slice(item);
    }

    let msg_len_bytes = msg_len.to_be_bytes();

    to[start..start + 4].clone_from_slice(&msg_len_bytes[..4]);
    to[start + 5..start + 7].clone_from_slice(&item_count.to_be_bytes());
}

pub fn write_map(to: &mut Vec<u8>, value: &DashMap<Vec<u8>, Vec<u8>>) {
    {
        // kind + 2 byte map size
        let mut response_len: u32 = 1 + 2;

        for item in value.iter() {
            let (key, value) = item.pair();

            // key len + key bytes len + value len + value bytes len
            response_len += 1 + key.len() as u32 + 4 + value.len() as u32;
        }

        to.extend_from_slice(&response_len.to_be_bytes());
    }

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
    {
        // kind + 2 byte set size
        let mut response_len: u32 = 1 + 2;

        for item in value.iter() {
            // item len + item bytes len
            response_len += 2 + item.len() as u32;
        }

        to.extend_from_slice(&response_len.to_be_bytes());
    }

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

    // kind + string len + string bytes len
    let response_len = 1 + 4 + len;
    to.extend_from_slice(&response_len.to_be_bytes());
    to.push(ResponseType::String as u8);

    to.extend_from_slice(&len.to_be_bytes());
    to.extend_from_slice(value.as_bytes());
}

pub fn write_value(to: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Boolean(boolean) => write_bool(to, *boolean),
        Value::Bytes(bytes) => write_bytes(to, bytes),
        Value::Float(float) => write_float(to, *float),
        Value::Integer(int) => write_int(to, *int),
        Value::List(list) => write_list(to, list),
        Value::Map(map) => write_map(to, map),
        Value::Set(set) => write_set(to, set),
        Value::String(string) => write_str(to, string),
    }
}

#[cfg(test)]
mod tests {
    use super::{Response, ResponseType};
    use crate::state::Value;
    use alloc::{borrow::ToOwned, string::String, vec::Vec};
    use core::{fmt::Debug, hash::Hash};
    use dashmap::{DashMap, DashSet};
    use static_assertions::assert_impl_all;

    assert_impl_all!(ResponseType: Clone, Copy, Debug, Eq, Hash, PartialEq);
    assert_impl_all!(
        Response: Debug,
        From<bool>,
        From<Vec<u8>>,
        From<f64>,
        From<i64>,
        From<Vec<Vec<u8>>>,
        From<DashMap<Vec<u8>, Vec<u8>>>,
        From<DashSet<Vec<u8>>>,
        From<String>,
    );

    #[test]
    fn test_bool() {
        assert_eq!(
            Response::from(false).as_bytes(),
            [0, 0, 0, 2, ResponseType::Boolean as u8, 0]
        );
        assert_eq!(
            Response::from(true).as_bytes(),
            [0, 0, 0, 2, ResponseType::Boolean as u8, 1]
        );
    }

    #[test]
    fn test_bytes() {
        assert_eq!(
            Response::from(b"hopdb".to_vec()).as_bytes(),
            [
                0,
                0,
                0,
                10,
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
                0,
                0,
                0,
                5,
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
                0,
                0,
                0,
                9,
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
            [
                0,
                0,
                0,
                9,
                ResponseType::Integer as u8,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                7
            ],
        );
        assert_eq!(
            Response::from(-7).as_bytes(),
            [
                0,
                0,
                0,
                9,
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
            [
                0,
                0,
                0,
                9,
                ResponseType::Integer as u8,
                0,
                0,
                0,
                0,
                0,
                1,
                10,
                29
            ],
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
                0,
                0,
                0,
                16,
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
                0,
                0,
                0,
                3,
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
                0,
                0,
                0,
                20,
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
                0,
                0,
                0,
                20,
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
            [0, 0, 0, 3, ResponseType::Map as u8, 0, 0]
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
                0,
                0,
                0,
                12,
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
                0,
                0,
                0,
                12,
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
            [0, 0, 0, 3, ResponseType::Set as u8, 0, 0]
        );
    }

    #[test]
    fn test_str() {
        assert_eq!(
            Response::from("foo bar baz".to_owned()).as_bytes(),
            [
                0,
                0,
                0,
                16,
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
            [0, 0, 0, 5, ResponseType::String as u8, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_response_as_bytes() {
        let resp = Response::Value(Value::Integer(3));
        assert_eq!(
            resp.as_bytes(),
            [
                0,
                0,
                0,
                9,
                ResponseType::Integer as u8,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                3,
            ]
        );
    }

    #[test]
    fn test_response_into_bytes() {
        let resp = Response::Value(Value::Integer(3));
        let mut buf = Vec::new();
        resp.copy_to(&mut buf);
        assert_eq!(
            buf,
            [
                0,
                0,
                0,
                9,
                ResponseType::Integer as u8,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                3,
            ]
        );
    }
}
