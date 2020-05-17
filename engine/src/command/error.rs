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
    KeyTypeUnexpected = 3,
    PreconditionFailed = 4,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::ArgumentRetrieval => f.write_str("couldn't retrieve required argument"),
            Self::KeyRetrieval => f.write_str("couldn't retrieve key"),
            Self::KeyTypeUnexpected => f.write_str("didn't expect a specified request key type"),
            Self::PreconditionFailed => f.write_str("a precondition for the command failed"),
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
            3 => Self::KeyTypeUnexpected,
            4 => Self::PreconditionFailed,
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use core::{
        convert::TryFrom,
        fmt::{Debug, Display},
        hash::Hash,
    };
    use static_assertions::assert_impl_all;

    assert_impl_all!(
        Error: Clone,
        Copy,
        Debug,
        Display,
        Eq,
        Hash,
        PartialEq,
        TryFrom<u8>
    );

    #[test]
    fn test_error_try_from_u8() {
        let variants = &[
            Error::ArgumentRetrieval,
            Error::KeyRetrieval,
            Error::WrongType,
            Error::KeyTypeUnexpected,
            Error::PreconditionFailed,
        ];

        for variant in variants {
            assert!(matches!(Error::try_from(*variant as u8), Ok(v) if v == *variant));
        }
    }
}
