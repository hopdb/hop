use std::result::Result as StdResult;

pub type Result<T, E = DispatchError> = StdResult<T, E>;

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DispatchError {
    BooleanValueInvalid = 0,
    KeyNonexistant = 1,
    KeyWrongType = 2,
    FloatTooSmall = 3,
    IntegerTooSmall = 4,
    TooFewArguments = 5,
}
