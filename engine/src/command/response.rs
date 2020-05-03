use alloc::vec::Vec;
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
pub struct Response(Vec<u8>);

impl Response {
    pub fn bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<bool> for Response {
    fn from(value: bool) -> Self {
        let mut bytes = Vec::with_capacity(2);
        bytes.push(ResponseType::Boolean as u8);
        bytes.push(if value { 1 } else { 0 });

        Self(bytes)
    }
}

impl From<&[u8]> for Response {
    fn from(value: &[u8]) -> Self {
        let mut bytes = Vec::new();
        bytes.push(ResponseType::Bytes as u8);
        let len = value.len() as u32;
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(value);

        Self(bytes)
    }
}

impl From<f64> for Response {
    fn from(value: f64) -> Self {
        let mut bytes = Vec::with_capacity(9);
        bytes.push(ResponseType::Float as u8);
        bytes.extend_from_slice(&value.to_be_bytes());

        Self(bytes)
    }
}

impl From<i64> for Response {
    fn from(value: i64) -> Self {
        let mut bytes = Vec::new();
        bytes.push(ResponseType::Integer as u8);
        bytes.extend_from_slice(&value.to_be_bytes());

        Self(bytes)
    }
}

impl From<&[Vec<u8>]> for Response {
    fn from(value: &[Vec<u8>]) -> Self {
        let mut bytes = Vec::new();
        bytes.push(ResponseType::List as u8);

        // The length of the list.
        bytes.extend_from_slice(&(value.len() as u16).to_be_bytes());

        // Now for each list item, push its length and then the item itself.
        for item in value {
            let len = item.len() as u32;

            bytes.extend_from_slice(&len.to_be_bytes());
            bytes.extend_from_slice(item);
        }

        Self(bytes)
    }
}

impl From<&DashMap<Vec<u8>, Vec<u8>>> for Response {
    fn from(value: &DashMap<Vec<u8>, Vec<u8>>) -> Self {
        let mut bytes = Vec::new();
        bytes.push(ResponseType::Map as u8);

        // Maps can only contain up to u16 items.
        bytes.extend_from_slice(&(value.len() as u16).to_be_bytes());

        for item in value.iter() {
            let (key, value) = item.pair();

            let key_len = key.len() as u8;
            let value_len = value.len() as u32;

            bytes.push(key_len);
            bytes.extend_from_slice(key);
            bytes.extend_from_slice(&value_len.to_be_bytes());
            bytes.extend_from_slice(value);
        }

        Self(bytes)
    }
}

impl From<&DashSet<Vec<u8>>> for Response {
    fn from(value: &DashSet<Vec<u8>>) -> Self {
        let mut bytes = Vec::new();
        bytes.push(ResponseType::Set as u8);

        // Sets can only contain up to u16 items.
        bytes.extend_from_slice(&(value.len() as u16).to_be_bytes());

        for item in value.iter() {
            let len = item.len() as u16;

            bytes.extend_from_slice(&len.to_be_bytes());
            bytes.extend_from_slice(item.key());
        }

        Self(bytes)
    }
}

impl From<&str> for Response {
    fn from(value: &str) -> Self {
        let mut bytes = Vec::new();
        bytes.push(ResponseType::String as u8);
        let len = value.len() as u32;

        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(value.as_bytes());

        Self(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::{Response, ResponseType};
    use alloc::vec::Vec;
    use dashmap::{DashMap, DashSet};

    #[test]
    fn test_bool() {
        assert_eq!(Response::from(false).0, [ResponseType::Boolean as u8, 0]);
        assert_eq!(Response::from(true).0, [ResponseType::Boolean as u8, 1]);
    }

    #[test]
    fn test_bytes() {
        assert_eq!(
            Response::from(b"hopdb".as_ref()).0,
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
            Response::from(b"".as_ref()).0,
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
            Response::from(7.4).0,
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
            Response::from(7).0,
            [ResponseType::Integer as u8, 0, 0, 0, 0, 0, 0, 0, 7],
        );
        assert_eq!(
            Response::from(-7).0,
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
            Response::from(68125).0,
            [ResponseType::Integer as u8, 0, 0, 0, 0, 0, 1, 10, 29],
        );
    }

    #[test]
    fn test_list() {
        let mut list = Vec::new();
        list.push(b"hop".to_vec());
        list.push(b"db".to_vec());

        assert_eq!(
            Response::from(list.as_slice()).0,
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
            Response::from(v.as_slice()).0,
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

        let resp = Response::from(&map).0;

        assert!(possible_values.iter().any(|v| v == resp.as_slice()));
    }

    #[test]
    fn test_map_empty() {
        assert_eq!(
            Response::from(&DashMap::new()).0,
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

        let resp = Response::from(&map).0;

        assert!(possible_values.iter().any(|v| v == resp.as_slice()));
    }

    #[test]
    fn test_set_empty() {
        assert_eq!(
            Response::from(&DashSet::new()).0,
            [ResponseType::Set as u8, 0, 0]
        );
    }

    #[test]
    fn test_str() {
        assert_eq!(
            Response::from("foo bar baz").0,
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
            Response::from("").0,
            [ResponseType::String as u8, 0, 0, 0, 0]
        );
    }
}
