use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::{state::Value, Hop};
use alloc::vec::Vec;

pub struct IncrementBy;

impl IncrementBy {
    pub fn increment_float_by(
        hop: &Hop,
        key: &[u8],
        amount: f64,
        resp: &mut Vec<u8>,
    ) -> DispatchResult<()> {
        let mut key = hop.state().key_or_insert_with(key, Value::integer);
        let float = key.as_float_mut().ok_or(DispatchError::KeyTypeDifferent)?;

        *float += amount;

        response::write_float(resp, *float);

        Ok(())
    }

    pub fn increment_int_by(
        hop: &Hop,
        key: &[u8],
        amount: i64,
        resp: &mut Vec<u8>,
    ) -> DispatchResult<()> {
        let mut key = hop.state().key_or_insert_with(key, Value::integer);
        let int = key
            .as_integer_mut()
            .ok_or(DispatchError::KeyTypeDifferent)?;

        *int += amount;

        response::write_int(resp, *int);

        Ok(())
    }
}

impl Dispatch for IncrementBy {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        if let Some(int) = req.typed_arg::<i64>(1) {
            Self::increment_int_by(hop, key, int, resp)
        } else if let Some(float) = req.typed_arg::<f64>(1) {
            Self::increment_float_by(hop, key, float, resp)
        } else {
            Err(DispatchError::ArgumentRetrieval)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IncrementBy;
    use crate::{
        command::{request::RequestBuilder, CommandId, Dispatch, DispatchError, Response},
        state::Value,
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_decrement_by() {
        let mut builder = RequestBuilder::new(CommandId::IncrementBy);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        assert!(builder.value(Value::Integer(3)).is_ok());
        let req = builder.into_request();
        let hop = Hop::new();
        let mut resp = Vec::new();

        assert!(IncrementBy::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(Response::from(3i64).as_bytes(), resp);
        assert_eq!(
            Some(&3),
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
            IncrementBy::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }

    #[test]
    fn test_no_amount() {
        let mut builder = RequestBuilder::new(CommandId::IncrementBy);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();

        let hop = Hop::new();
        let mut resp = Vec::new();

        assert_eq!(
            DispatchError::ArgumentRetrieval,
            IncrementBy::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }
}
