use alloc::vec::Vec;
use core::convert::TryInto;
use super::state::error::{DispatchError, Result};

pub fn into_boolean(byte: u8) -> Result<bool> {
    Ok(if byte == 1 {
        true
    } else if byte == 0 {
        false
    } else {
        return Err(DispatchError::BooleanValueInvalid);
    })
}

pub fn into_float(bytes: &[u8]) -> Result<f64> {
    let arr = bytes.try_into().map_err(|_| DispatchError::FloatTooSmall)?;

    Ok(f64::from_be_bytes(arr))
}

pub fn into_integer(bytes: &[u8]) -> Result<i64> {
    let arr = bytes.try_into().map_err(|_| DispatchError::IntegerTooSmall)?;

    Ok(i64::from_be_bytes(arr))
}

pub fn reserve<T>(list: &mut Vec<T>, additional: usize) {
    #[cfg(nightly)]
    {
        let _ = list.try_reserve(additional);
    }

    #[cfg(not(nightly))]
    {
        let _ = list.reserve(additional);
    }
}

#[cfg(test)]
mod tests {
    use crate::state::error::{DispatchError, Result};
    use super::*;

    #[test]
    fn test_into_boolean() -> Result<()> {
        assert_eq!(into_boolean(0)?, false);
        assert_eq!(into_boolean(1)?, true);

        (2 ..= u8::max_value()).for_each(|i| {
            assert_eq!(into_boolean(i).unwrap_err(), DispatchError::BooleanValueInvalid);
        });

        Ok(())
    }

    #[test]
    fn test_reserve() {
        let mut foo: Vec<u8> = Vec::with_capacity(0);
        assert_eq!(foo.capacity(), 0);
        reserve(&mut foo, 12);
        assert_eq!(foo.capacity(), 12);

        // Capacity is already enough, so it does nothing.
        reserve(&mut foo, 11);
        assert_eq!(foo.capacity(), 12);
    }
}
