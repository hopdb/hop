use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::{
    state::{KeyType, Value},
    Hop,
};
use alloc::vec::Vec;

pub struct DecrementBy;

impl DecrementBy {
    pub fn decrement_float_by(
        hop: &Hop,
        key: &[u8],
        amount: f64,
        resp: &mut Vec<u8>,
    ) -> DispatchResult<()> {
        let mut key = hop.state().key_or_insert_with(key, Value::float);
        let float = key.as_float_mut().ok_or(DispatchError::KeyUnspecified)?;

        *float -= amount as f64;

        response::write_float(resp, *float);

        Ok(())
    }

    pub fn decrement_int_by(
        hop: &Hop,
        key: &[u8],
        amount: i64,
        resp: &mut Vec<u8>,
    ) -> DispatchResult<()> {
        let mut key = hop.state().key_or_insert_with(key, Value::integer);
        let int = key.as_integer_mut().ok_or(DispatchError::KeyUnspecified)?;

        *int -= amount;

        response::write_int(resp, *int);

        Ok(())
    }

    pub fn decrement(hop: &Hop, key: &[u8], resp: &mut Vec<u8>) -> DispatchResult<()> {
        hop.state().key_or_insert_with(b"foo", Value::integer);

        match hop.state().key_ref(key).map(|r| r.value().kind()) {
            Some(KeyType::Float) => Self::decrement_float_by(hop, key, 1f64, resp),
            Some(KeyType::Integer) => Self::decrement_int_by(hop, key, 1, resp),
            _ => Err(DispatchError::KeyTypeDifferent),
        }
    }
}

impl Dispatch for DecrementBy {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        if let Some(int) = req.typed_arg::<i64>(1) {
            Self::decrement_int_by(hop, key, int, resp)
        } else if let Some(float) = req.typed_arg::<f64>(1) {
            Self::decrement_float_by(hop, key, float, resp)
        } else {
            Err(DispatchError::ArgumentRetrieval)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DecrementBy;
    use crate::{
        command::{request::RequestBuilder, CommandId, Dispatch, DispatchError, Response},
        state::Value,
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_decrement_by() {
        let mut builder = RequestBuilder::new(CommandId::DecrementBy);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.value(Value::Integer(3)).is_ok());
        let req = builder.into_request();
        let hop = Hop::new();
        let mut resp = Vec::new();

        assert!(DecrementBy::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(Response::from(-3i64).as_bytes(), resp);
        assert_eq!(
            Some(&-3),
            hop.state()
                .key_ref(b"foo")
                .as_deref()
                .and_then(Value::as_integer_ref)
        );
    }

    #[test]
    fn test_no_key() {
        let req = RequestBuilder::new(CommandId::Decrement).into_request();
        let hop = Hop::new();
        let mut resp = Vec::new();

        assert_eq!(
            DispatchError::KeyUnspecified,
            DecrementBy::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }

    #[test]
    fn test_no_amount() {
        let mut builder = RequestBuilder::new(CommandId::DecrementBy);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();
        let hop = Hop::new();
        let mut resp = Vec::new();

        assert_eq!(
            DispatchError::ArgumentRetrieval,
            DecrementBy::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }
}
