use super::{
    super::{Dispatch, DispatchError, DispatchResult, Request},
    increment_by::IncrementBy,
};
use crate::Hop;
use alloc::vec::Vec;

pub struct DecrementBy;

impl Dispatch for DecrementBy {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        if let Some(int) = req.typed_arg::<i64>(1) {
            IncrementBy::increment_int_by(hop, key, 0 - int, resp)
        } else if let Some(float) = req.typed_arg::<f64>(1) {
            IncrementBy::increment_float_by(hop, key, 0f64 - float, resp)
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
