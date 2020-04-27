use core::result::Result as CoreResult;

pub type Result<T, E = RetrievalError> = CoreResult<T, E>;

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum RetrievalError {
    Nonexistant = 0,
    TypeWrong = 1,
}
