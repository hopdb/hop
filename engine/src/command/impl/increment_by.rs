use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::{
    state::{
        object::{Float, Integer},
        KeyType,
    },
    Hop,
};
use alloc::vec::Vec;

pub struct IncrementBy;

impl IncrementBy {
    pub fn increment(
        hop: &Hop,
        key: &[u8],
        key_type: Option<KeyType>,
        amount: i64,
        resp: &mut Vec<u8>,
    ) -> DispatchResult<()> {
        match key_type {
            Some(KeyType::Integer) | None => {
                let mut int = hop
                    .state()
                    .typed_key::<Integer>(key)
                    .ok_or(DispatchError::KeyUnspecified)?;

                *int += amount;

                response::write_int(resp, *int)
            }
            Some(KeyType::Float) => {
                let mut float = hop
                    .state()
                    .typed_key::<Float>(key)
                    .ok_or(DispatchError::KeyUnspecified)?;

                *float += amount as f64;

                response::write_float(resp, *float)
            }
            Some(_) => return Err(DispatchError::KeyTypeInvalid),
        }

        Ok(())
    }
}

impl Dispatch for IncrementBy {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        Self::increment(hop, key, req.key_type(), 1, resp)
    }
}
