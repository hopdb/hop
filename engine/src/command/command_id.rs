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
pub struct InvalidCommandId;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CommandId {
    Increment = 0,
    Decrement = 1,
    IncrementBy = 2,
    DecrementBy = 3,
    Append = 20,
    Length = 21,
    Echo = 100,
    Stats = 101,
}

impl CommandId {
    pub fn argument_notation(self) -> ArgumentNotation {
        use ArgumentNotation::*;
        use CommandId::*;

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
        use CommandId::*;

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

impl Display for CommandId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

impl FromStr for CommandId {
    type Err = InvalidCommandId;

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
            _ => return Err(InvalidCommandId),
        })
    }
}

impl TryFrom<u8> for CommandId {
    type Error = InvalidCommandId;

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
            _ => return Err(InvalidCommandId),
        })
    }
}

impl<'a> TryFrom<&'a str> for CommandId {
    type Error = InvalidCommandId;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::{ArgumentNotation, CommandId, InvalidCommandId};
    use core::{
        convert::TryFrom,
        fmt::{Debug, Display},
        hash::Hash,
        str::FromStr,
    };
    use static_assertions::assert_impl_all;

    assert_impl_all!(ArgumentNotation: Clone, Debug, Eq, PartialEq);
    assert_impl_all!(
        CommandId: Clone,
        Copy,
        Debug,
        Display,
        FromStr,
        Eq,
        Hash,
        PartialEq,
        PartialOrd,
        Ord,
        TryFrom<u8>,
        TryFrom<&'static str>,
    );
    assert_impl_all!(InvalidCommandId: Debug);

    #[test]
    fn test_from_str() {
        assert_eq!(CommandId::Append, CommandId::from_str("append").unwrap());
        assert_eq!(
            CommandId::DecrementBy,
            CommandId::from_str("decrement:by").unwrap()
        );
        assert_eq!(
            CommandId::Decrement,
            CommandId::from_str("decrement").unwrap()
        );
        assert_eq!(CommandId::Echo, CommandId::from_str("echo").unwrap());
        assert_eq!(
            CommandId::IncrementBy,
            CommandId::from_str("increment:by").unwrap()
        );
        assert_eq!(
            CommandId::Increment,
            CommandId::from_str("increment").unwrap()
        );
        assert_eq!(CommandId::Stats, CommandId::from_str("stats").unwrap());
        assert_eq!(CommandId::Length, CommandId::from_str("length").unwrap());
    }

    #[test]
    fn test_try_from_u8() {
        assert_eq!(CommandId::Append, CommandId::try_from(20).unwrap());
        assert_eq!(CommandId::DecrementBy, CommandId::try_from(3).unwrap());
        assert_eq!(CommandId::Decrement, CommandId::try_from(1).unwrap());
        assert_eq!(CommandId::Echo, CommandId::try_from(100).unwrap());
        assert_eq!(CommandId::IncrementBy, CommandId::try_from(2).unwrap());
        assert_eq!(CommandId::Increment, CommandId::try_from(0).unwrap());
        assert_eq!(CommandId::Stats, CommandId::try_from(101).unwrap());
        assert_eq!(CommandId::Length, CommandId::try_from(21).unwrap());
    }

    #[test]
    fn test_name() {
        assert_eq!("append", CommandId::Append.name());
        assert_eq!("decrement:by", CommandId::DecrementBy.name());
        assert_eq!("decrement", CommandId::Decrement.name());
        assert_eq!("echo", CommandId::Echo.name());
        assert_eq!("increment:by", CommandId::IncrementBy.name());
        assert_eq!("increment", CommandId::Increment.name());
        assert_eq!("stats", CommandId::Stats.name());
        assert_eq!("length", CommandId::Length.name());
    }
}
