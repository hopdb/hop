use hop_lib::command::{CommandId, Request};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{BufRead, Error as IoError},
    str::FromStr,
};

#[derive(Debug)]
pub enum InputError {
    InvalidCommandType { provided_name: String },
    NoCommandProvided,
    Retrieval { source: IoError },
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::InvalidCommandType { provided_name } => {
                write!(f, "the command '{}' is invalid", provided_name)
            }
            Self::NoCommandProvided => f.write_str("no command was provided"),
            Self::Retrieval { .. } => f.write_str("failed to retrieve a line of input"),
        }
    }
}

impl Error for InputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidCommandType { .. } => None,
            Self::NoCommandProvided => None,
            Self::Retrieval { source } => Some(source),
        }
    }
}

pub fn process_command(reader: &mut impl BufRead, buf: &mut String) -> Result<Request, InputError> {
    reader
        .read_line(buf)
        .map_err(|source| InputError::Retrieval { source })?;
    let buf = buf.trim();

    let mut split = buf.split(' ');

    let cmd_name = match split.next() {
        Some(cmd_name) if !cmd_name.is_empty() => cmd_name,
        _ => {
            return Err(InputError::NoCommandProvided);
        }
    };

    let cmd_type = CommandId::from_str(cmd_name).map_err(|_| InputError::InvalidCommandType {
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

#[cfg(test)]
mod tests {
    use super::InputError;
    use hop_lib::command::CommandId;
    use std::{error::Error, io::Cursor};

    #[test]
    fn test_process_command_echo() -> Result<(), Box<dyn Error>> {
        let mut reader = Cursor::new("echo abc");
        let mut buf = String::new();

        let req = super::process_command(&mut reader, &mut buf)?;

        assert_eq!(req.kind(), CommandId::Echo);
        assert_eq!(req.arg(0), Some(&b"abc".to_vec()));

        Ok(())
    }

    #[test]
    fn test_process_command_echo_no_args() -> Result<(), Box<dyn Error>> {
        let mut reader = Cursor::new("echo");
        let mut buf = String::new();

        let req = super::process_command(&mut reader, &mut buf)?;

        assert_eq!(req.kind(), CommandId::Echo);
        assert!(req.args().is_none());

        Ok(())
    }

    #[test]
    fn test_invalid_command() -> Result<(), Box<dyn Error>> {
        let name = "invalidcommand".to_owned();

        let mut reader = Cursor::new(format!("{} foo", name));
        let mut buf = String::new();

        let res = super::process_command(&mut reader, &mut buf).unwrap_err();

        if let InputError::InvalidCommandType { provided_name } = res {
            assert_eq!(provided_name, name);
        } else {
            assert!(false);
        }

        Ok(())
    }
}
