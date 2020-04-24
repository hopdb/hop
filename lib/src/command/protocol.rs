use crate::pool::Pool;
use std::{
    convert::{TryFrom, TryInto},
    mem,
};
use super::CommandType;

#[derive(Clone, Debug)]
pub struct CommandInfo {
    pub arguments: Option<Vec<Vec<u8>>>,
    pub kind: CommandType,
}

#[derive(Debug)]
pub enum ParseError {
    CommandTypeInvalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Stage {
    Init,
    Kind(CommandType),
    ArgumentParsing {
        argument_count: u8,
        kind: CommandType,
    },
}

impl Default for Stage {
    fn default() -> Self {
        Stage::Init
    }
}

#[derive(Debug)]
pub struct Context {
    argument_pool: Pool<Vec<u8>>,
    buf_args: Option<Vec<Vec<u8>>>,
    idx: usize,
    stage: Stage,
}

impl Context {
    pub fn new() -> Self {
        Self {
            argument_pool: Pool::new(1, Vec::new),
            buf_args: Some(Vec::new()),
            idx: 0,
            stage: Stage::default(),
        }
    }

    pub fn feed(&mut self, buf: &mut Vec<u8>) -> Result<Option<CommandInfo>, ParseError> {
        loop {
            // We need to do this check on the first iteration to make sure we
            // were actually given *any* data, and after each iteration to make
            // sure that there's more data to process.
            if buf.is_empty() {
                return Ok(None);
            }

            match self.stage {
                Stage::Init => {
                    let byte = buf[0];
                    let kind = CommandType::try_from(byte).map_err(|_| ParseError::CommandTypeInvalid)?;

                    // If the command type is simple and has no arguments or keys, then
                    // we can just return a successful command here.
                    if kind.is_simple() {
                        self.reset_light();

                        return Ok(Some(CommandInfo {
                            arguments: None,
                            kind,
                        }));
                    }

                    self.stage = Stage::Kind(kind);
                    self.idx += 1;
                },
                Stage::Kind(kind) => {
                    let argument_count = buf[self.idx];

                    self.stage = Stage::ArgumentParsing {
                        argument_count,
                        kind,
                    };
                    self.idx += 1;
                },
                Stage::ArgumentParsing { argument_count, .. } => {
                    // panic!("{} {:?}", self.idx, buf);
                    let len_bytes = buf[self.idx..self.idx + 4].try_into().unwrap();
                    let arg_len = u32::from_be_bytes(len_bytes) as usize;

                    if buf[self.idx..].len() < arg_len as usize {
                        return Ok(None);
                    }

                    let arg = &buf[self.idx..self.idx + arg_len];
                    let mut owned_arg = self.argument_pool.pull();
                    owned_arg.extend_from_slice(arg);
                    self.buf_args.as_mut().unwrap().push(owned_arg);

                    self.idx += 4 + arg_len;

                    if self.buf_args.as_mut().unwrap().len() as u8 == argument_count {
                        break;
                    }
                },
            }
        }

        let stage = mem::replace(&mut self.stage, Stage::default());

        if let Stage::ArgumentParsing { kind, .. } = stage {
            let args = if !self.buf_args.as_ref().unwrap().is_empty() {
                self.buf_args.take()
            } else {
                None
            };

            return Ok(Some(CommandInfo {
                arguments: args,
                kind,
            }));
        } else {
            unreachable!();
        }
    }

    pub fn reset(&mut self, mut args: Vec<Vec<u8>>) {
        self.reset_light();

        args.drain(..).for_each(|mut vec| {
            vec.clear();

            self.argument_pool.push(vec);
        });

        self.buf_args.replace(args);
    }

    fn reset_light(&mut self) {
        self.idx = 0;
        self.stage = Stage::default();
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use super::{super::CommandType, Context};

    #[test]
    fn test_increment_foo() -> Result<(), Box<dyn Error>> {
        // - the first byte is the command type's ID (up to ID 255)
        // - the second byte, if the command type has arguments, is the number of
        //   arguments (up to 255)
        // - for each argument, the first 4 bytes is the length of the argument
        //   in bytes (so, each argument can theoretically be up to 4032MiB in
        //   size)
        // - the rest of the bytes, up to the defined length of the argument,
        //   is the argument
        let mut cmd = [
            // command type 0 is "increment int"
            0,
            // there is 1 argument
            1,
            // the argument has a length of 3 bytes
            0, 0, 0, 3,
            // the argument is 'foo'
            b'f', b'o', b'o',
        ].to_vec();

        // the context might not be given all of the data upfront (eg large
        // streams of data, just because of how tcp works, etc.), so often you
        // need to feed multiple rounds of data into it to get a complete
        // command request
        let mut ctx = Context::new();
        // but here we're feeding in all the data in one go
        let cmd = ctx.feed(&mut cmd).unwrap().unwrap();
        assert_eq!(cmd.kind, CommandType::IncrementInt);

        Ok(())
    }
}
