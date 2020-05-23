use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};
use hop_engine::{
    command::{
        command_id::{CommandId, KeyNotation},
        Request,
    },
    state::KeyType,
};
use std::error::Error;

#[derive(Debug)]
pub enum ParseError {
    ArgumentInvalid { argument: String, key_type: KeyType },
    InvalidCommandType { provided_name: String },
    KeyUnspecified,
    MapIncomplete { key: String },
    NoCommandProvided,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::ArgumentInvalid { argument, key_type } => f.write_fmt(format_args!(
                "the argument '{}' is not a {}",
                argument,
                key_type_name(*key_type)
            )),
            Self::InvalidCommandType { provided_name } => {
                f.write_fmt(format_args!("the command '{}' is invalid", provided_name))
            }
            Self::KeyUnspecified => {
                f.write_str("A key is required for this command but none was provided")
            }
            Self::MapIncomplete { key } => f.write_fmt(format_args!(
                "the map has an incomplete key-value; the last key was {}",
                key
            )),
            Self::NoCommandProvided => f.write_str("no command was provided"),
        }
    }
}

impl Error for ParseError {}

pub fn parse(input: &str) -> Result<Request, ParseError> {
    let mut split = input.split(' ');

    let cmd_name = match split.next() {
        Some(cmd_name) if !cmd_name.is_empty() => cmd_name,
        _ => {
            return Err(ParseError::NoCommandProvided);
        }
    };

    let (cmd_id, key_type) = command(cmd_name).ok_or_else(|| ParseError::InvalidCommandType {
        provided_name: cmd_name.to_owned(),
    })?;

    let mut arg_iter = split.peekable();
    let args = if arg_iter.peek().is_some() {
        let key_type = key_type.unwrap_or(KeyType::Bytes);

        Some(input_args(arg_iter, cmd_id, key_type)?)
    } else {
        None
    };

    Ok(if let Some(key_type) = key_type {
        Request::new_with_type(cmd_id, args, key_type)
    } else {
        Request::new(cmd_id, args)
    })
}

/// Iterate over the provided string input arguments and convert them to
/// request-ready arguments.
///
/// # Errors
///
/// Returns [`ParseError::ArgumentInvalid`] when the argument is not the same
/// type as the provided key type. For example, when a float key type is
/// provided but the argument is "abc", which clearly can't be parsed into a
/// float.
fn input_args<'a>(
    provided: impl IntoIterator<Item = &'a str>,
    cmd_id: CommandId,
    key_type: KeyType,
) -> Result<Vec<Vec<u8>>, ParseError> {
    let mut args = Vec::new();

    let mut iter = provided.into_iter();

    match cmd_id.key_notation() {
        KeyNotation::None => {}
        KeyNotation::One => {
            let arg = iter.next().ok_or_else(|| ParseError::KeyUnspecified)?;

            args.push(arg.as_bytes().to_vec());
        }
        KeyNotation::Two => {
            for _ in 0..2 {
                let arg = iter.next().ok_or_else(|| ParseError::KeyUnspecified)?;

                args.push(arg.as_bytes().to_vec());
            }
        }
        KeyNotation::Multiple => {
            // Since all of the arguments are keys, we can just return the input
            // directly.
            return Ok(iter.map(|key| key.as_bytes().to_vec()).collect());
        }
    }

    while let Some(arg) = iter.next() {
        match key_type {
            KeyType::Boolean => {
                let boolean = arg
                    .parse::<bool>()
                    .map_err(|_| ParseError::ArgumentInvalid {
                        argument: arg.to_owned(),
                        key_type: KeyType::Boolean,
                    })?;

                args.push(vec![boolean as u8]);
            }
            KeyType::Bytes => args.push(arg.as_bytes().to_vec()),
            KeyType::Float => {
                let float = arg
                    .parse::<f64>()
                    .map_err(|_| ParseError::ArgumentInvalid {
                        argument: arg.to_owned(),
                        key_type: KeyType::Float,
                    })?;

                args.push(float.to_be_bytes().to_vec());
            }
            KeyType::Integer => {
                let int = arg
                    .parse::<i64>()
                    .map_err(|_| ParseError::ArgumentInvalid {
                        argument: arg.to_owned(),
                        key_type: KeyType::Integer,
                    })?;

                args.push(int.to_be_bytes().to_vec());
            }
            KeyType::List => args.push(arg.as_bytes().to_vec()),
            KeyType::Map => {
                let value = match iter.next() {
                    Some(arg) => arg.as_bytes().to_vec(),
                    None => break,
                };

                args.push(arg.as_bytes().to_vec());
                args.push(value);
            }
            KeyType::Set => {
                args.push(arg.as_bytes().to_vec());
            }
            KeyType::String => {
                args.push(arg.as_bytes().to_vec());
            }
        }
    }

    Ok(args)
}

fn command(name: &str) -> Option<(CommandId, Option<KeyType>)> {
    if let Ok(cmd_id) = CommandId::from_str(name) {
        return Some((cmd_id, None));
    }

    let mut parts = name.rsplitn(2, ':');
    let key_type = parts.next().and_then(key_type)?;
    let cmd_id = parts
        .next()
        .and_then(|name| CommandId::from_str(name).ok())?;

    Some((cmd_id, Some(key_type)))
}

fn key_type(key_type: &str) -> Option<KeyType> {
    Some(match key_type {
        "boolean" | "bool" => KeyType::Boolean,
        "bytes" => KeyType::Bytes,
        "float" => KeyType::Float,
        "integer" | "int" => KeyType::Integer,
        "list" => KeyType::List,
        "map" => KeyType::Map,
        "set" => KeyType::Set,
        "string" | "str" => KeyType::String,
        _ => return None,
    })
}

fn key_type_name(key_type: KeyType) -> &'static str {
    match key_type {
        KeyType::Boolean => "bool",
        KeyType::Bytes => "bytes",
        KeyType::Float => "float",
        KeyType::Integer => "int",
        KeyType::List => "list",
        KeyType::Map => "map",
        KeyType::Set => "set",
        KeyType::String => "str",
    }
}

#[cfg(test)]
mod tests {
    use hop_engine::{command::CommandId, state::KeyType};

    #[test]
    fn test_args() {
        assert_eq!(
            vec![vec![1u8]],
            super::input_args("true".split(' '), CommandId::Echo, KeyType::Boolean).unwrap()
        );
    }

    #[test]
    fn test_command() {
        assert_eq!(Some((CommandId::Echo, None)), super::command("echo"));
        assert_eq!(
            Some((CommandId::Increment, Some(KeyType::Float))),
            super::command("increment:float")
        );
        assert_eq!(
            Some((CommandId::Increment, Some(KeyType::Integer))),
            super::command("increment:int")
        );
    }

    #[test]
    fn test_command_is_int() {
        let req = super::parse("is:int foo bar").unwrap();
        assert_eq!(CommandId::Is, req.kind());
        assert_eq!(Some(KeyType::Integer), req.key_type());
        assert_eq!(Some(b"foo".as_ref()), req.arg(0));
        assert_eq!(Some(b"bar".as_ref()), req.arg(1));
        assert!(req.arg(2).is_none());
    }

    #[test]
    fn test_command_invalid_key_type() {
        assert!(super::command("increment:floatt").is_none());
        assert!(super::command("increment:").is_none());
    }

    #[test]
    fn test_key_type() {
        assert_eq!(Some(KeyType::Boolean), super::key_type("boolean"));
        assert_eq!(Some(KeyType::Boolean), super::key_type("bool"));
        assert_eq!(Some(KeyType::Bytes), super::key_type("bytes"));
        assert_eq!(Some(KeyType::Float), super::key_type("float"));
        assert_eq!(Some(KeyType::Integer), super::key_type("integer"));
        assert_eq!(Some(KeyType::Integer), super::key_type("int"));
        assert_eq!(Some(KeyType::List), super::key_type("list"));
        assert_eq!(Some(KeyType::Map), super::key_type("map"));
        assert_eq!(Some(KeyType::Set), super::key_type("set"));
        assert_eq!(Some(KeyType::String), super::key_type("string"));
        assert_eq!(Some(KeyType::String), super::key_type("str"));
    }

    #[test]
    fn test_key_type_name() {
        assert_eq!(super::key_type_name(KeyType::Boolean), "bool");
        assert_eq!(super::key_type_name(KeyType::Bytes), "bytes");
        assert_eq!(super::key_type_name(KeyType::Float), "float");
        assert_eq!(super::key_type_name(KeyType::Integer), "int");
        assert_eq!(super::key_type_name(KeyType::List), "list");
        assert_eq!(super::key_type_name(KeyType::Map), "map");
        assert_eq!(super::key_type_name(KeyType::Set), "set");
        assert_eq!(super::key_type_name(KeyType::String), "str");
    }

    #[test]
    fn test_parse() {
        let req = super::parse("echo").unwrap();
        assert_eq!(CommandId::Echo, req.kind());
        assert!(req.args(..).is_none());

        let req = super::parse("increment:int").unwrap();
        assert_eq!(CommandId::Increment, req.kind());
        assert_eq!(Some(KeyType::Integer), req.key_type());

        let req = super::parse("increment:by:int foo 3").unwrap();
        assert_eq!(CommandId::IncrementBy, req.kind());
        assert_eq!(Some(KeyType::Integer), req.key_type());
        assert!(req.args(..).is_some());
        assert_eq!(2, req.arg_count());
        assert_eq!(Some([b'f', b'o', b'o'].as_ref()), req.arg(0));
        assert_eq!(Some(3i64.to_be_bytes().as_ref()), req.arg(1));
    }
}
