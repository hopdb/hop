use alloc::str::FromStr;
use core::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentNotation {
    Multiple,
    None,
    One,
}

#[derive(Debug)]
pub struct InvalidCommandType;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CommandType {
    Increment = 0,
    Decrement = 1,
    IncrementBy = 2,
    DecrementBy = 3,
    Append = 20,
    Length = 21,
    Echo = 100,
    Stats = 101,
}

impl CommandType {
    pub fn argument_notation(self) -> ArgumentNotation {
        use ArgumentNotation::*;
        use CommandType::*;

        match self {
            Append => One,
            Echo => Multiple,
            Increment => None,
            IncrementBy => One,
            Decrement => None,
            DecrementBy => One,
            Stats => None,
            Length => One,
        }
    }

    pub fn has_key(self) -> bool {
        use CommandType::*;

        match self {
            Echo | Stats => false,
            _ => true,
        }
    }

    pub fn is_simple(self) -> bool {
        self.argument_notation() == ArgumentNotation::None && !self.has_key()
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Append => "append",
            Self::DecrementBy => "decrement:by",
            Self::Decrement => "decrement",
            Self::Echo => "echo",
            Self::IncrementBy => "increment:by",
            Self::Increment => "increment",
            Self::Stats => "stats",
            Self::Length => "length",
        }
    }
}

impl Display for CommandType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

impl FromStr for CommandType {
    type Err = InvalidCommandType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "append" => Self::Append,
            "decrement:by" => Self::DecrementBy,
            "decrement" => Self::Decrement,
            "echo" => Self::Echo,
            "increment:by" => Self::IncrementBy,
            "increment" => Self::Increment,
            "stats" => Self::Stats,
            "length" => Self::Length,
            _ => return Err(InvalidCommandType),
        })
    }
}

impl TryFrom<u8> for CommandType {
    type Error = InvalidCommandType;

    fn try_from(num: u8) -> Result<Self, Self::Error> {
        Ok(match num {
            0 => Self::Increment,
            1 => Self::Decrement,
            2 => Self::IncrementBy,
            3 => Self::DecrementBy,
            20 => Self::Append,
            21 => Self::Length,
            100 => Self::Echo,
            101 => Self::Stats,
            _ => return Err(InvalidCommandType),
        })
    }
}

impl<'a> TryFrom<&'a str> for CommandType {
    type Error = InvalidCommandType;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::CommandType;
    use core::{convert::TryFrom, str::FromStr};

    #[test]
    fn test_from_str() {
        assert_eq!(
            CommandType::Append,
            CommandType::from_str("append").unwrap()
        );
        assert_eq!(
            CommandType::DecrementBy,
            CommandType::from_str("decrement:by").unwrap()
        );
        assert_eq!(
            CommandType::Decrement,
            CommandType::from_str("decrement").unwrap()
        );
        assert_eq!(CommandType::Echo, CommandType::from_str("echo").unwrap());
        assert_eq!(
            CommandType::IncrementBy,
            CommandType::from_str("increment:by").unwrap()
        );
        assert_eq!(
            CommandType::Increment,
            CommandType::from_str("increment").unwrap()
        );
        assert_eq!(CommandType::Stats, CommandType::from_str("stats").unwrap());
        assert_eq!(
            CommandType::Length,
            CommandType::from_str("length").unwrap()
        );
    }

    #[test]
    fn test_try_from_u8() {
        assert_eq!(CommandType::Append, CommandType::try_from(20).unwrap());
        assert_eq!(CommandType::DecrementBy, CommandType::try_from(3).unwrap());
        assert_eq!(CommandType::Decrement, CommandType::try_from(1).unwrap());
        assert_eq!(CommandType::Echo, CommandType::try_from(100).unwrap());
        assert_eq!(CommandType::IncrementBy, CommandType::try_from(2).unwrap());
        assert_eq!(CommandType::Increment, CommandType::try_from(0).unwrap());
        assert_eq!(CommandType::Stats, CommandType::try_from(101).unwrap());
        assert_eq!(
            CommandType::Length,
            CommandType::try_from(21).unwrap()
        );
    }

    #[test]
    fn test_name() {
        assert_eq!("append", CommandType::Append.name());
        assert_eq!("decrement:by", CommandType::DecrementBy.name());
        assert_eq!("decrement", CommandType::Decrement.name());
        assert_eq!("echo", CommandType::Echo.name());
        assert_eq!("increment:by", CommandType::IncrementBy.name());
        assert_eq!("increment", CommandType::Increment.name());
        assert_eq!("stats", CommandType::Stats.name());
        assert_eq!("length", CommandType::Length.name());
    }
}
