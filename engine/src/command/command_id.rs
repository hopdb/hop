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
    Set = 10,
    Delete = 12,
    Exists = 13,
    Is = 14,
    Rename = 15,
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
            Delete => One,
            Echo => Multiple,
            Exists => Multiple,
            Increment => None,
            IncrementBy => One,
            Is => Multiple,
            Decrement => None,
            DecrementBy => One,
            Length => One,
            Rename => One,
            Set => Multiple,
            Stats => None,
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
            Self::Delete => "delete",
            Self::Echo => "echo",
            Self::Exists => "exists",
            Self::IncrementBy => "increment:by",
            Self::Increment => "increment",
            Self::Is => "is",
            Self::Length => "length",
            Self::Rename => "rename",
            Self::Set => "set",
            Self::Stats => "stats",
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
            "delete" => Self::Delete,
            "echo" => Self::Echo,
            "exists" => Self::Exists,
            "increment:by" => Self::IncrementBy,
            "increment" => Self::Increment,
            "is" => Self::Is,
            "length" => Self::Length,
            "rename" => Self::Rename,
            "set" => Self::Set,
            "stats" => Self::Stats,
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
            10 => Self::Set,
            12 => Self::Delete,
            13 => Self::Exists,
            14 => Self::Is,
            15 => Self::Rename,
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
        assert_eq!(CommandId::Delete, CommandId::from_str("delete").unwrap());
        assert_eq!(CommandId::Echo, CommandId::from_str("echo").unwrap());
        assert_eq!(CommandId::Exists, CommandId::from_str("exists").unwrap());
        assert_eq!(
            CommandId::IncrementBy,
            CommandId::from_str("increment:by").unwrap()
        );
        assert_eq!(
            CommandId::Increment,
            CommandId::from_str("increment").unwrap()
        );
        assert_eq!(CommandId::Is, CommandId::from_str("is").unwrap());
        assert_eq!(CommandId::Length, CommandId::from_str("length").unwrap());
        assert_eq!(CommandId::Rename, CommandId::from_str("rename").unwrap());
        assert_eq!(CommandId::Set, CommandId::from_str("set").unwrap());
        assert_eq!(CommandId::Stats, CommandId::from_str("stats").unwrap());
    }

    #[test]
    fn test_try_from_u8() {
        assert_eq!(CommandId::Append, CommandId::try_from(20).unwrap());
        assert_eq!(CommandId::DecrementBy, CommandId::try_from(3).unwrap());
        assert_eq!(CommandId::Decrement, CommandId::try_from(1).unwrap());
        assert_eq!(CommandId::Delete, CommandId::try_from(12).unwrap());
        assert_eq!(CommandId::Echo, CommandId::try_from(100).unwrap());
        assert_eq!(CommandId::Exists, CommandId::try_from(13).unwrap());
        assert_eq!(CommandId::IncrementBy, CommandId::try_from(2).unwrap());
        assert_eq!(CommandId::Increment, CommandId::try_from(0).unwrap());
        assert_eq!(CommandId::Is, CommandId::from_str("is").unwrap());
        assert_eq!(CommandId::Length, CommandId::try_from(21).unwrap());
        assert_eq!(CommandId::Rename, CommandId::try_from(15).unwrap());
        assert_eq!(CommandId::Set, CommandId::try_from(10).unwrap());
        assert_eq!(CommandId::Stats, CommandId::try_from(101).unwrap());
    }

    #[test]
    fn test_name() {
        assert_eq!("append", CommandId::Append.name());
        assert_eq!("decrement:by", CommandId::DecrementBy.name());
        assert_eq!("decrement", CommandId::Decrement.name());
        assert_eq!("delete", CommandId::Delete.name());
        assert_eq!("echo", CommandId::Echo.name());
        assert_eq!("exists", CommandId::Exists.name());
        assert_eq!("increment:by", CommandId::IncrementBy.name());
        assert_eq!("increment", CommandId::Increment.name());
        assert_eq!("is", CommandId::Is.name());
        assert_eq!("length", CommandId::Length.name());
        assert_eq!("rename", CommandId::Rename.name());
        assert_eq!("set", CommandId::Set.name());
        assert_eq!("stats", CommandId::Stats.name());
    }
}
