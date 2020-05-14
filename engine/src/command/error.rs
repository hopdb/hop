use core::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
    result::Result as CoreResult,
};

pub type Result<T, E = Error> = CoreResult<T, E>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Error {
    ArgumentRetrieval = 0,
    KeyRetrieval = 1,
    WrongType = 2,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::ArgumentRetrieval => f.write_str("couldn't retrieve required argument"),
            Self::KeyRetrieval => f.write_str("couldn't retrieve key"),
            Self::WrongType => f.write_str("the key has the wrong type"),
        }
    }
}

impl TryFrom<u8> for Error {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::ArgumentRetrieval,
            1 => Self::KeyRetrieval,
            2 => Self::WrongType,
            _ => return Err(()),
        })
    }
}
