use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};
use hop_engine::command::{CommandId, Request};
use std::error::Error;

#[derive(Debug)]
pub enum ParseError {
    InvalidCommandType { provided_name: String },
    NoCommandProvided,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::InvalidCommandType { provided_name } => {
                write!(f, "the command '{}' is invalid", provided_name)
            }
            Self::NoCommandProvided => f.write_str("no command was provided"),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidCommandType { .. } => None,
            Self::NoCommandProvided => None,
        }
    }
}

pub fn parse(input: &str) -> Result<Request, ParseError> {
    let mut split = input.split(' ');

    let cmd_name = match split.next() {
        Some(cmd_name) if !cmd_name.is_empty() => cmd_name,
        _ => {
            return Err(ParseError::NoCommandProvided);
        }
    };

    let cmd_type = CommandId::from_str(cmd_name).map_err(|_| ParseError::InvalidCommandType {
        provided_name: cmd_name.to_owned(),
    })?;

    let mut arg_iter = split.peekable();
    let args = if arg_iter.peek().is_some() {
        Some(arg_iter.map(|s| s.as_bytes().to_vec()).collect())
    } else {
        None
    };

    Ok(Request::new(cmd_type, args))
}
