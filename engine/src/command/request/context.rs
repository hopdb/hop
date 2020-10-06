use super::{super::ContextConclusion, Request};
use crate::{command::CommandId, state::KeyType};
use alloc::borrow::Cow;
use arrayvec::ArrayVec;
use core::convert::{TryFrom, TryInto};

type Conclusion<'a> = ContextConclusion<(CommandId, Option<KeyType>)>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ParseError {
    CommandIdInvalid = 0,
    KeyTypeInvalid = 1,
}

impl TryFrom<u8> for ParseError {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::CommandIdInvalid,
            1 => Self::KeyTypeInvalid,
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Stage {
    Init,
    Kind {
        command_id: CommandId,
        key_type: Option<KeyType>,
    },
    ArgumentParsing {
        argument_count: u8,
        command_id: CommandId,
        key_type: Option<KeyType>,
    },
}

impl Default for Stage {
    fn default() -> Self {
        Stage::Init
    }
}

#[derive(Debug)]
pub struct Context {
    idx: usize,
    positions: ArrayVec<[usize; 256]>,
    stage: Stage,
}

impl Context {
    const ARG_LEN_BYTES: usize = 4;

    pub fn new() -> Self {
        Default::default()
    }

    pub fn feed<'a>(&'a mut self, buf: &'a [u8]) -> Result<Option<Request<'a>>, ParseError> {
        loop {
            let conclusion = {
                // We need to do this check on the first iteration to make sure we
                // were actually given *any* data, and after each iteration to make
                // sure that there's more data to process.
                let idx = self.idx;

                if buf.get(idx..).is_none() {
                    return Ok(None);
                }

                match self.stage {
                    Stage::Init => self.stage_init(buf)?,
                    Stage::Kind {
                        command_id,
                        key_type,
                    } => self.stage_kind(buf, key_type, command_id)?,
                    Stage::ArgumentParsing {
                        argument_count,
                        command_id,
                        key_type,
                    } => self.stage_argument_parsing(buf, command_id, key_type, argument_count)?,
                }
            };

            match conclusion {
                Conclusion::Finished((command_id, key_type)) => {
                    self.reset();

                    return Ok(Some(Request {
                        buf: Cow::Borrowed(buf),
                        command_id,
                        key_type,
                        positions: Cow::Borrowed(&self.positions),
                    }));
                }
                Conclusion::Incomplete => return Ok(None),
                Conclusion::Next => continue,
            }
        }
    }

    fn stage_init<'a>(&'a mut self, buf: &'a [u8]) -> Result<Conclusion<'a>, ParseError> {
        let byte = match buf.first() {
            Some(byte) => *byte,
            None => return Ok(Conclusion::Incomplete),
        };

        // If the first bit is flipped, then this byte is denoting the type of
        // key to work with. This means that byte idx 2 is the argument length.
        //
        // If the first bit is 0, then this byte is the argument length, and the
        // type of key to work with is not a requirement.
        let key_type = if byte >> 7 == 1 {
            let key_type_id = byte >> 1;

            Some(KeyType::try_from(key_type_id).map_err(|_| ParseError::KeyTypeInvalid)?)
        } else {
            None
        };

        let command_id = CommandId::try_from(byte).map_err(|_| ParseError::CommandIdInvalid)?;

        // If the command type is simple and has no arguments or keys, then
        // we can just return a successful command here.
        if command_id.is_simple() {
            return Ok(Conclusion::Finished((command_id, None)));
        }

        self.stage = Stage::Kind {
            command_id,
            key_type,
        };
        self.idx = self.idx.wrapping_add(1);

        Ok(Conclusion::Next)
    }

    fn stage_kind(
        &mut self,
        buf: &[u8],
        key_type: Option<KeyType>,
        command_id: CommandId,
    ) -> Result<Conclusion, ParseError> {
        let argument_count = match buf.get(self.idx) {
            Some(argument_count) => *argument_count,
            None => return Ok(Conclusion::Incomplete),
        };

        self.stage = Stage::ArgumentParsing {
            argument_count,
            command_id,
            key_type,
        };
        self.idx = self.idx.saturating_add(1);

        Ok(Conclusion::Next)
    }

    fn stage_argument_parsing<'a>(
        &'a mut self,
        buf: &'a [u8],
        command_id: CommandId,
        key_type: Option<KeyType>,
        argument_count: u8,
    ) -> Result<Conclusion, ParseError> {
        let len_bytes = match buf.get(self.idx..self.idx + Self::ARG_LEN_BYTES) {
            Some(bytes) => bytes.try_into().unwrap(),
            None => return Ok(Conclusion::Incomplete),
        };

        let arg_len = u32::from_be_bytes(len_bytes) as usize;

        if buf.get(self.idx..self.idx + arg_len).is_some() {
            self.positions.push(self.idx + arg_len);
        } else {
            return Ok(Conclusion::Incomplete);
        }

        self.idx += 4 + arg_len;

        if self.positions.len() == argument_count as usize {
            Ok(Conclusion::Finished((command_id, key_type)))
        } else {
            Ok(Conclusion::Next)
        }
    }

    fn reset(&mut self) {
        self.idx = 0;
        self.positions.clear();
        self.stage = Stage::default();
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            idx: 0,
            positions: ArrayVec::new(),
            stage: Stage::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{super::error::Result, CommandId},
        Context, ParseError, Stage,
    };
    use core::{convert::TryFrom, fmt::Debug, hash::Hash};
    use static_assertions::assert_impl_all;

    assert_impl_all!(Context: Debug, Default);
    assert_impl_all!(
        ParseError: Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        PartialEq,
        TryFrom<u8>
    );
    assert_impl_all!(Stage: Clone, Debug, Default, Eq, PartialEq);

    #[test]
    fn test_increment_foo() -> Result<()> {
        // - the first byte is the command type's ID (up to ID 255)
        // - the second byte, if the command type has arguments, is the number of
        //   arguments (up to 255)
        // - for each argument, the first 4 bytes is the length of the argument
        //   in bytes (so, each argument can theoretically be up to 4032MiB in
        //   size)
        // - the rest of the bytes, up to the defined length of the argument,
        //   is the argument
        let cmd = [
            // command type 0 is "increment int"
            0, // there is 1 argument
            1, // the argument has a length of 3 bytes
            0, 0, 0, 3, // the argument is 'foo'
            b'f', b'o', b'o',
        ]
        .to_vec();

        // the context might not be given all of the data upfront (eg large
        // streams of data, just because of how tcp works, etc.), so often you
        // need to feed multiple rounds of data into it to get a complete
        // command request
        let mut ctx = Context::new();
        // but here we're feeding in all the data in one go
        let cmd = ctx
            .feed(&cmd)
            .expect("parses correctly")
            .expect("returns a command");

        assert_eq!(cmd.command_id, CommandId::Increment);

        Ok(())
    }

    #[test]
    fn test_parse_error_try_from_u8() {
        assert_eq!(
            ParseError::try_from(0).unwrap(),
            ParseError::CommandIdInvalid
        );
        assert_eq!(ParseError::try_from(1).unwrap(), ParseError::KeyTypeInvalid);
    }
}
