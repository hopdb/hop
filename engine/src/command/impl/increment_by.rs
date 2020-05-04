use super::super::{DispatchError, DispatchResult, Dispatch, Request, Response};
use crate::{
    state::{
        object::{Float, Integer},
        KeyType,
    },
    Hop,
};

pub struct IncrementBy;

impl IncrementBy {
    pub fn increment(
        hop: &Hop,
        key: &[u8],
        key_type: Option<KeyType>,
        amount: i64,
    ) -> DispatchResult<Response> {
        match key_type {
            Some(KeyType::Integer) | None => {
                let mut int = hop
                    .state()
                    .typed_key::<Integer>(key)
                    .ok_or(DispatchError::KeyRetrieval)?;

                *int += amount;

                Ok(Response::from(*int))
            }
            Some(KeyType::Float) => {
                let mut float = hop
                    .state()
                    .typed_key::<Float>(key)
                    .ok_or(DispatchError::KeyRetrieval)?;

                *float += amount as f64;

                Ok(Response::from(*float))
            }
            Some(_) => Err(DispatchError::WrongType),
        }
    }
}

impl Dispatch for IncrementBy {
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Response> {
        let key = req.key().ok_or(DispatchError::KeyRetrieval)?;

        Self::increment(hop, key, req.key_type(), 1)
    }
}
