use super::ArgumentNotation;
use alloc::str::FromStr;
use core::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub struct InvalidCommandType;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CommandType {
    IncrementInt = 0,
    DecrementInt = 1,
    IncrementIntBy = 2,
    DecrementIntBy = 3,
    Append = 20,
    StringLength = 21,
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
            IncrementInt => None,
            IncrementIntBy => One,
            DecrementInt => None,
            DecrementIntBy => One,
            Stats => None,
            StringLength => One,
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
            Self::DecrementIntBy => "decrement:int:by",
            Self::DecrementInt => "decrement:int",
            Self::Echo => "echo",
            Self::IncrementIntBy => "increment:int:by",
            Self::IncrementInt => "increment:int",
            Self::Stats => "stats",
            Self::StringLength => "string:length",
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
            "decrement:int:by" => Self::DecrementIntBy,
            "decrement:int" => Self::DecrementInt,
            "echo" => Self::Echo,
            "increment:int:by" => Self::IncrementIntBy,
            "increment:int" => Self::IncrementInt,
            "stats" => Self::Stats,
            "string:length" => Self::StringLength,
            _ => return Err(InvalidCommandType),
        })
    }
}

impl TryFrom<u8> for CommandType {
    type Error = InvalidCommandType;

    fn try_from(num: u8) -> Result<Self, Self::Error> {
        Ok(match num {
            0 => Self::IncrementInt,
            1 => Self::DecrementInt,
            2 => Self::IncrementIntBy,
            3 => Self::DecrementIntBy,
            20 => Self::Append,
            21 => Self::StringLength,
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
    use core::{
        convert::TryFrom,
        str::FromStr,
    };
    use super::CommandType;

    #[test]
    fn test_from_str() {
        assert_eq!(CommandType::Append, CommandType::from_str("append").unwrap());
        assert_eq!(CommandType::DecrementIntBy, CommandType::from_str("decrement:int:by").unwrap());
        assert_eq!(CommandType::DecrementInt, CommandType::from_str("decrement:int").unwrap());
        assert_eq!(CommandType::Echo, CommandType::from_str("echo").unwrap());
        assert_eq!(CommandType::IncrementIntBy, CommandType::from_str("increment:int:by").unwrap());
        assert_eq!(CommandType::IncrementInt, CommandType::from_str("increment:int").unwrap());
        assert_eq!(CommandType::Stats, CommandType::from_str("stats").unwrap());
        assert_eq!(CommandType::StringLength, CommandType::from_str("string:length").unwrap());
    }

    #[test]
    fn test_try_from_u8() {
        assert_eq!(CommandType::Append, CommandType::try_from(20).unwrap());
        assert_eq!(CommandType::DecrementIntBy, CommandType::try_from(3).unwrap());
        assert_eq!(CommandType::DecrementInt, CommandType::try_from(1).unwrap());
        assert_eq!(CommandType::Echo, CommandType::try_from(100).unwrap());
        assert_eq!(CommandType::IncrementIntBy, CommandType::try_from(2).unwrap());
        assert_eq!(CommandType::IncrementInt, CommandType::try_from(0).unwrap());
        assert_eq!(CommandType::Stats, CommandType::try_from(101).unwrap());
        assert_eq!(CommandType::StringLength, CommandType::try_from(21).unwrap());
    }

    #[test]
    fn test_name() {
        assert_eq!("append", CommandType::Append.name());
        assert_eq!("decrement:int:by", CommandType::DecrementIntBy.name());
        assert_eq!("decrement:int", CommandType::DecrementInt.name());
        assert_eq!("echo", CommandType::Echo.name());
        assert_eq!("increment:int:by", CommandType::IncrementIntBy.name());
        assert_eq!("increment:int", CommandType::IncrementInt.name());
        assert_eq!("stats", CommandType::Stats.name());
        assert_eq!("string:length", CommandType::StringLength.name());
    }
}
