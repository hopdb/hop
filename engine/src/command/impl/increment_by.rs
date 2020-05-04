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
    ) -> DispatchResult<Vec<u8>> {
        match key_type {
            Some(KeyType::Integer) | None => {
                let mut int = hop
                    .state()
                    .typed_key::<Integer>(key)
                    .ok_or(DispatchError::KeyRetrieval)?;

                *int += amount;

                Ok(response::write_int(*int))
            }
            Some(KeyType::Float) => {
                let mut float = hop
                    .state()
                    .typed_key::<Float>(key)
                    .ok_or(DispatchError::KeyRetrieval)?;

                *float += amount as f64;

                Ok(response::write_float(*float))
            }
            Some(_) => Err(DispatchError::WrongType),
        }
    }
}

impl Dispatch for IncrementBy {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Vec<u8>> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        Self::increment(hop, key, req.key_type(), 1)
    }
}
