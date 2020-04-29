use super::prelude::*;
use crate::state::{object::{Float, Integer}, KeyType};

pub struct DecrementBy;

impl DecrementBy {
    pub fn decrement(hop: &Hop, key: &[u8], key_type: Option<KeyType>, amount: i64) -> Result<Response> {
        match key_type {
            Some(KeyType::Integer) | None => {
                let mut int = hop.state().typed_key::<Integer>(key).ok_or(Error::KeyRetrieval)?;

                *int -= amount;

                Ok(Response::from(*int))
            },
            Some(KeyType::Float) => {
                let mut float = hop.state().typed_key::<Float>(key).ok_or(Error::KeyRetrieval)?;

                *float -= amount as f64;

                Ok(Response::from(*float))
            },
            Some(_) => Err(Error::WrongType),
        }
    }
}

impl Dispatch for DecrementBy {
    fn dispatch(hop: &Hop, req: &Request) -> Result<Response> {
        let key = req.key().ok_or(Error::KeyRetrieval)?;

        Self::decrement(hop, key, req.key_type(), 1)
    }
}
