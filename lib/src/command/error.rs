use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    result::Result as CoreResult,
};

pub type Result<T, E = Error> = CoreResult<T, E>;

#[derive(Debug)]
pub enum Error {
    ArgumentRetrieval,
    KeyRetrieval,
    WrongType,
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
