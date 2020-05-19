// Refer to the request context for in-depth documentation on how these contexts
// work.

use super::{Response, ResponseType};
use crate::{
    command::{request::ParseError as RequestParseError, DispatchError},
    state::Value,
};
use alloc::{string::String, vec::Vec};
use core::{
    convert::{TryFrom, TryInto},
    mem,
};
use dashmap::{DashMap, DashSet};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ParseError {
    DispatchErrorInvalid,
    ParseErrorInvalid,
    /// The payload is too large. The command should have been sent as multiple
    /// appends.
    ///
    /// The state of the connection is now unknown due to an unknown amount of
    /// data and possible list commands, so the connection needs to be
    /// re-initiated. The session can be resumed.
    PayloadTooLarge,
    ResponseTypeInvalid,
    /// The string isn't valid UTF-8.
    StringInvalid,
}

#[derive(Debug)]
pub enum Instruction {
    Concluded(Response),
    ReadBytes(usize),
}

#[derive(Clone, Debug)]
enum Stage {
    Init,
    /// The type is known, and now the length of the argument(s) is being read
    /// for the following types:
    ///
    /// - bytes
    /// - list
    /// - map
    /// - set
    /// - string
    TypeInit {
        kind: ResponseType,
        read_len: usize,
    },
    Boolean,
    Bytes {
        len: u32,
    },
    Float,
    Integer,
    List {
        args: Vec<Vec<u8>>,
        len: u16,
    },
    Map {
        map: DashMap<Vec<u8>, Vec<u8>>,
        len: u16,
    },
    DispatchError,
    ParseError,
    Set {
        args: DashSet<Vec<u8>>,
        len: u16,
    },
    String {
        len: u32,
    },
}

impl Default for Stage {
    fn default() -> Self {
        Self::Init
    }
}

#[derive(Debug, Default)]
pub struct Context {
    idx: usize,
    stage: Stage,
}

impl Context {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn feed(&mut self, buf: &[u8]) -> Result<Instruction, ParseError> {
        loop {
            let instruction = match self.stage {
                Stage::Init => self.stage_init(buf)?,
                Stage::Boolean => self.stage_boolean(buf)?,
                Stage::Bytes { len } => self.stage_bytes(buf, len)?,
                Stage::Float => self.stage_float(buf)?,
                Stage::Integer => self.stage_integer(buf)?,
                Stage::List { .. } => self.stage_list(buf)?,
                Stage::Map { .. } => self.stage_map(buf)?,
                Stage::Set { .. } => self.stage_set(buf)?,
                Stage::String { len } => self.stage_string(buf, len)?,
                Stage::TypeInit { kind, read_len } => self.stage_type_init(buf, kind, read_len)?,
                Stage::DispatchError => self.stage_dispatch_error(buf)?,
                Stage::ParseError => self.stage_parse_error(buf)?,
            };

            match instruction {
                Some(Instruction::Concluded(response)) => {
                    self.reset();

                    return Ok(Instruction::Concluded(response));
                }
                Some(Instruction::ReadBytes(amount)) => return Ok(Instruction::ReadBytes(amount)),
                None => continue,
            }
        }
    }

    fn stage_init(&mut self, buf: &[u8]) -> Result<Option<Instruction>, ParseError> {
        debug_assert!(self.idx == 0);

        let byte = match buf.get(0) {
            Some(byte) => *byte,
            None => return Ok(Some(Instruction::ReadBytes(1))),
        };

        let kind = ResponseType::try_from(byte).map_err(|_| ParseError::ResponseTypeInvalid)?;

        self.stage = match kind {
            ResponseType::Boolean => Stage::Boolean,
            ResponseType::Float => Stage::Float,
            ResponseType::Integer => Stage::Integer,
            ResponseType::List | ResponseType::Map | ResponseType::Set => {
                Stage::TypeInit { kind, read_len: 2 }
            }
            ResponseType::Bytes | ResponseType::String => Stage::TypeInit { kind, read_len: 4 },
            ResponseType::DispatchError => Stage::DispatchError,
            ResponseType::ParseError => Stage::ParseError,
        };

        self.idx += 1;

        Ok(None)
    }

    fn stage_boolean(&mut self, buf: &[u8]) -> Result<Option<Instruction>, ParseError> {
        debug_assert_eq!(self.idx, 1);

        let byte = match buf.get(self.idx) {
            Some(byte) => *byte,
            None => return Ok(Some(Instruction::ReadBytes(1))),
        };

        Ok(Some(Instruction::Concluded(Response::from(byte != 0))))
    }

    fn stage_bytes(&mut self, buf: &[u8], len: u32) -> Result<Option<Instruction>, ParseError> {
        debug_assert_eq!(self.idx, 5);
        debug_assert_ne!(len, 0);

        let bytes = match buf.get(self.idx..self.idx + len as usize) {
            Some(bytes) => bytes,
            None => {
                let buf_len = buf.len();
                let remaining = remaining_bytes(self.idx, buf_len, len as usize);

                return Ok(Some(Instruction::ReadBytes(remaining)));
            }
        };

        Ok(Some(Instruction::Concluded(Response::from(bytes.to_vec()))))
    }

    fn stage_dispatch_error(&mut self, buf: &[u8]) -> Result<Option<Instruction>, ParseError> {
        debug_assert_eq!(self.idx, 1);

        let byte = match buf.get(self.idx) {
            Some(byte) => *byte,
            None => return Ok(Some(Instruction::ReadBytes(1))),
        };

        let variant = DispatchError::try_from(byte).unwrap();

        Ok(Some(Instruction::Concluded(Response::from(variant))))
    }

    fn stage_float(&mut self, buf: &[u8]) -> Result<Option<Instruction>, ParseError> {
        debug_assert_eq!(self.idx, 1);

        let bytes = match buf.get(self.idx..self.idx + 8) {
            Some(bytes) => bytes,
            None => return Ok(Some(Instruction::ReadBytes(1))),
        };

        let float = f64::from_be_bytes(bytes.try_into().unwrap());

        Ok(Some(Instruction::Concluded(Response::from(float))))
    }

    fn stage_integer(&mut self, buf: &[u8]) -> Result<Option<Instruction>, ParseError> {
        debug_assert_eq!(self.idx, 1);

        let bytes = match buf.get(self.idx..self.idx + 8) {
            Some(bytes) => bytes,
            None => return Ok(Some(Instruction::ReadBytes(1))),
        };

        let int = i64::from_be_bytes(bytes.try_into().unwrap());

        Ok(Some(Instruction::Concluded(Response::from(int))))
    }

    fn stage_list(&mut self, buf: &[u8]) -> Result<Option<Instruction>, ParseError> {
        debug_assert!(self.idx > 2);
        debug_assert!(buf.len() > 2);

        let arg_size_end = self.idx + 4;

        let arg_len = match buf.get(self.idx..arg_size_end) {
            Some(arg_len) => u32::from_be_bytes(arg_len.try_into().unwrap()),
            None => {
                let remaining = remaining_bytes(self.idx, buf.len(), 4);

                return Ok(Some(Instruction::ReadBytes(remaining)));
            }
        };

        let arg_value_end = arg_size_end + arg_len as usize;

        let arg = match buf.get(arg_size_end..arg_value_end) {
            Some(arg) => arg,
            None => {
                let remaining = arg_value_end - buf.len();

                return Ok(Some(Instruction::ReadBytes(remaining)));
            }
        };

        match self.stage {
            Stage::List { ref mut args, len } => {
                args.push(arg.to_vec());

                if args.len() < len as usize {
                    self.idx = arg_value_end;

                    return Ok(None);
                }
            }
            _ => unreachable!(),
        };

        match mem::take(&mut self.stage) {
            Stage::List { args, .. } => Ok(Some(Instruction::Concluded(Response::from(args)))),
            _ => unreachable!(),
        }
    }

    fn stage_map(&mut self, buf: &[u8]) -> Result<Option<Instruction>, ParseError> {
        debug_assert!(self.idx > 2);
        debug_assert!(buf.len() > 2);

        let key_size_end = self.idx + 1;

        let key_len = match buf.get(self.idx..key_size_end) {
            Some(key_len) => u8::from_be_bytes(key_len.try_into().unwrap()),
            None => {
                let remaining = remaining_bytes(self.idx, buf.len(), 1);

                return Ok(Some(Instruction::ReadBytes(remaining)));
            }
        };

        let key_value_end = key_size_end + key_len as usize;

        let key = match buf.get(key_size_end..key_value_end) {
            Some(key) => key,
            None => {
                let remaining = remaining_bytes(key_size_end, buf.len(), key_value_end);

                return Ok(Some(Instruction::ReadBytes(remaining)));
            }
        };

        let value_size_end = key_value_end + 4;

        let value_len = match buf.get(key_value_end..value_size_end) {
            Some(value_len) => u32::from_be_bytes(value_len.try_into().unwrap()),
            None => {
                let remaining = remaining_bytes(self.idx, buf.len(), value_size_end);

                return Ok(Some(Instruction::ReadBytes(remaining)));
            }
        };

        let value_end = value_size_end + value_len as usize;

        let value = match buf.get(value_size_end..value_end) {
            Some(value) => value,
            None => {
                let remaining = remaining_bytes(value_size_end, buf.len(), value_end);

                return Ok(Some(Instruction::ReadBytes(remaining)));
            }
        };

        match self.stage {
            Stage::Map { ref map, len } => {
                map.insert(key.to_vec(), value.to_vec());

                if map.len() < len as usize {
                    self.idx = value_end;

                    return Ok(None);
                }
            }
            _ => unreachable!(),
        };

        match mem::take(&mut self.stage) {
            Stage::Map { map, .. } => Ok(Some(Instruction::Concluded(Response::from(map)))),
            _ => unreachable!(),
        }
    }

    fn stage_parse_error(&mut self, buf: &[u8]) -> Result<Option<Instruction>, ParseError> {
        debug_assert_eq!(self.idx, 1);

        let byte = match buf.get(self.idx) {
            Some(byte) => *byte,
            None => return Ok(Some(Instruction::ReadBytes(1))),
        };

        let variant = RequestParseError::try_from(byte).unwrap();

        Ok(Some(Instruction::Concluded(Response::from(variant))))
    }

    fn stage_type_init(
        &mut self,
        buf: &[u8],
        kind: ResponseType,
        read_len: usize,
    ) -> Result<Option<Instruction>, ParseError> {
        debug_assert_eq!(self.idx, 1);

        let bytes = match buf.get(self.idx..self.idx + read_len) {
            Some(bytes) => bytes,
            None => return Ok(Some(Instruction::ReadBytes(read_len + 1 - buf.len()))),
        };

        debug_assert_eq!(bytes.len(), read_len);

        self.stage = match kind {
            ResponseType::Bytes => {
                let len = u32::from_be_bytes(bytes.try_into().unwrap());

                Stage::Bytes { len }
            }
            ResponseType::List => {
                let len = u16::from_be_bytes(bytes.try_into().unwrap());

                Stage::List {
                    args: Vec::new(),
                    len,
                }
            }
            ResponseType::Map => {
                let len = u16::from_be_bytes(bytes.try_into().unwrap());

                if len == 0 {
                    return Ok(Some(Instruction::Concluded(Response::from(DashMap::new()))));
                }

                Stage::Map {
                    len,
                    map: DashMap::new(),
                }
            }
            ResponseType::Set => {
                let len = u16::from_be_bytes(bytes.try_into().unwrap());

                Stage::Set {
                    args: DashSet::new(),
                    len,
                }
            }
            ResponseType::String => {
                let len = u32::from_be_bytes(bytes.try_into().unwrap());

                if len == 0 {
                    return Ok(Some(Instruction::Concluded(Response::Value(
                        Value::string(),
                    ))));
                }

                Stage::String { len }
            }
            // These are handled as unique branches.
            ResponseType::Boolean
            | ResponseType::DispatchError
            | ResponseType::Float
            | ResponseType::Integer
            | ResponseType::ParseError => {
                unreachable!();
            }
        };

        self.idx = self
            .idx
            .checked_add(read_len)
            .ok_or(ParseError::PayloadTooLarge)?;

        Ok(None)
    }

    fn stage_set(&mut self, buf: &[u8]) -> Result<Option<Instruction>, ParseError> {
        debug_assert!(self.idx > 2);
        debug_assert!(buf.len() > 2);

        let arg_size_end = self.idx + 2;

        let arg_len = match buf.get(self.idx..arg_size_end) {
            Some(arg_len) => u16::from_be_bytes(arg_len.try_into().unwrap()),
            None => {
                let remaining = remaining_bytes(self.idx, buf.len(), 2);

                return Ok(Some(Instruction::ReadBytes(remaining)));
            }
        };

        let arg_value_end = arg_size_end + arg_len as usize;

        let arg = match buf.get(arg_size_end..arg_value_end) {
            Some(arg) => arg,
            None => {
                let remaining = remaining_bytes(arg_size_end, buf.len(), arg_value_end);

                return Ok(Some(Instruction::ReadBytes(remaining)));
            }
        };

        match self.stage {
            Stage::Set { ref args, len } => {
                args.insert(arg.to_vec());

                if args.len() < len as usize {
                    self.idx = arg_value_end;

                    return Ok(None);
                }
            }
            _ => unreachable!(),
        };

        // Our parsing here is done, so to return the argument set we need to
        // swap out the stage so that we can *move* the arguments.
        match mem::take(&mut self.stage) {
            Stage::Set { args, .. } => Ok(Some(Instruction::Concluded(Response::from(args)))),
            _ => unreachable!(),
        }
    }

    fn stage_string(
        &mut self,
        buf: &[u8],
        read_len: u32,
    ) -> Result<Option<Instruction>, ParseError> {
        let read_len = read_len as usize;

        debug_assert!(self.idx > 0);
        debug_assert!(read_len > 0);

        let bytes = match buf.get(self.idx..self.idx + read_len) {
            Some(bytes) => bytes,
            None => {
                return Ok(Some(Instruction::ReadBytes(
                    self.idx + read_len - buf.len(),
                )))
            }
        };

        let string = String::from_utf8(bytes.to_vec()).map_err(|_| ParseError::StringInvalid)?;

        Ok(Some(Instruction::Concluded(Response::from(string))))
    }

    fn reset(&mut self) {
        self.idx = 0;
        self.stage = Stage::default();
    }
}

fn remaining_bytes(start: usize, read: usize, to_read: usize) -> usize {
    debug_assert!(start > 0);
    debug_assert!(read <= start.saturating_add(to_read));

    start.saturating_add(to_read).saturating_sub(read)
}

#[cfg(test)]
mod tests {
    use super::{
        super::{Response, ResponseType},
        Context, Instruction, ParseError, Stage,
    };
    use crate::{command::DispatchError, state::Value};
    use core::{fmt::Debug, hash::Hash};
    use static_assertions::assert_impl_all;

    assert_impl_all!(Context: Debug, Default);
    assert_impl_all!(Instruction: Debug);
    assert_impl_all!(ParseError: Clone, Copy, Debug, Eq, Hash, PartialEq);
    assert_impl_all!(Stage: Clone, Debug, Default);

    #[test]
    fn test_resets_automatically() {
        let mut ctx = Context::new();

        ctx.feed(&[ResponseType::Boolean as u8, 1]).unwrap();
        assert_eq!(ctx.idx, 0);
        assert!(matches!(ctx.stage, Stage::Init));
    }

    #[test]
    fn test_boolean() {
        let mut ctx = Context::new();

        let res = ctx.feed(&[ResponseType::Boolean as u8]);
        assert!(matches!(res, Ok(Instruction::ReadBytes(1))));

        let res = ctx.feed(&[ResponseType::Boolean as u8, 1]);
        assert!(matches!(
            res,
            Ok(Instruction::Concluded(Response::Value(Value::Boolean(
                true
            ))))
        ));
    }

    #[test]
    fn test_bytes() {
        let mut ctx = Context::new();
        let mut buf = [1u8, 0, 0, 0, 3].to_vec();
        assert!(matches!(ctx.feed(&buf), Ok(Instruction::ReadBytes(3))));
        buf.push(2);
        buf.push(3);
        assert!(matches!(ctx.feed(&buf), Ok(Instruction::ReadBytes(1))));
        buf.push(0);
        assert!(matches!(
            ctx.feed(&buf),
            Ok(Instruction::Concluded(Response::Value(Value::Bytes(x)))) if x == [2, 3, 0]),);
    }

    #[test]
    fn test_req_dispatch_error_unfinished() {
        let mut ctx = Context::new();
        let buf = [ResponseType::DispatchError as u8];
        assert!(matches!(ctx.feed(&buf), Ok(Instruction::ReadBytes(1))));
    }

    #[test]
    fn test_req_dispatch_error_argument_retrieval() {
        let mut ctx = Context::new();
        let buf = [
            ResponseType::DispatchError as u8,
            DispatchError::ArgumentRetrieval as u8,
        ];
        assert!(matches!(
            ctx.feed(&buf),
            Ok(Instruction::Concluded(Response::DispatchError(
                DispatchError::ArgumentRetrieval
            )))
        ));
    }

    #[test]
    fn test_req_dispatch_error_key_retrieval() {
        let mut ctx = Context::new();
        let buf = [
            ResponseType::DispatchError as u8,
            DispatchError::KeyRetrieval as u8,
        ];
        assert!(matches!(
            ctx.feed(&buf),
            Ok(Instruction::Concluded(Response::DispatchError(
                DispatchError::KeyRetrieval
            )))
        ));
    }

    #[test]
    fn test_req_dispatch_error_wrong_type() {
        let mut ctx = Context::new();
        let buf = [
            ResponseType::DispatchError as u8,
            DispatchError::WrongType as u8,
        ];
        assert!(matches!(
            ctx.feed(&buf),
            Ok(Instruction::Concluded(Response::DispatchError(
                DispatchError::WrongType
            )))
        ));
    }

    #[test]
    fn test_remaining_bytes() {
        assert_eq!(super::remaining_bytes(5, 5, 4), 4);
        assert_eq!(super::remaining_bytes(5, 7, 4), 2);
    }

    #[test]
    fn test_list() {
        let mut ctx = Context::new();
        let buf = [
            ResponseType::List as u8,
            // list items
            0,
            2,
            // item 1 len
            0,
            0,
            0,
            3,
            // arg 1 value
            b'f',
            b'o',
            b'o',
            // arg 2 len
            0,
            0,
            0,
            3,
            b'b',
            b'a',
            b'r',
        ]
        .to_vec();
        assert!(matches!(
            ctx.feed(&buf),
            Ok(Instruction::Concluded(Response::Value(Value::List(list)))) if list == &[b"foo", b"bar"]));
    }

    #[test]
    fn test_map() {
        let mut ctx = Context::new();
        let buf = [
            ResponseType::Map as u8,
            // item count
            0,
            1,
            // item 1 key len
            3,
            // item 1 key
            b'f',
            b'o',
            b'o',
            // item 1 value len
            0,
            0,
            0,
            4,
            // item 1 value
            b'b',
            b'a',
            b'r',
            b'!',
        ]
        .to_vec();
        assert!(
            matches!(ctx.feed(&buf), Ok(Instruction::Concluded(Response::Value(Value::Map(map)))) if map.len() == 1)
        );
    }

    #[test]
    fn test_map_no_items() {
        let mut ctx = Context::new();
        let buf = [ResponseType::Map as u8, 0, 0];
        assert!(
            matches!(ctx.feed(&buf), Ok(Instruction::Concluded(Response::Value(Value::Map(map)))) if map.is_empty())
        );
    }

    #[test]
    fn test_set() {
        let mut ctx = Context::new();
        let buf = [
            ResponseType::Set as u8,
            // set len
            0,
            2,
            // arg 1 len
            0,
            2,
            // arg 1 value
            5,
            3,
            // arg 2 len (incomplete)
            0,
        ]
        .to_vec();
        assert!(matches!(ctx.feed(&buf), Ok(Instruction::ReadBytes(1))));
    }

    #[test]
    fn test_string() {
        let mut ctx = Context::new();

        let mut input = [ResponseType::String as u8, 0, 0, 0, 3, b'a'].to_vec();

        let res = ctx.feed(input.as_slice());
        assert!(matches!(res, Ok(Instruction::ReadBytes(2))));

        input.push(b'b');
        let res = ctx.feed(input.as_slice());
        assert!(matches!(res, Ok(Instruction::ReadBytes(1))));

        input.push(b'c');
        let res = ctx.feed(input.as_slice());
        assert!(
            matches!(res, Ok(Instruction::Concluded(Response::Value(Value::String(x)))) if x == "abc")
        );

        assert_eq!(ctx.idx, 0);
    }
}
