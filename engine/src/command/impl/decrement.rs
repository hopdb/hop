use super::{
    super::{Dispatch, DispatchError, DispatchResult, Request},
    decrement_by::DecrementBy,
};
use crate::Hop;
use alloc::vec::Vec;

pub struct Decrement;

impl Dispatch for Decrement {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        DecrementBy::decrement(hop, key, resp)
    }
}

#[cfg(test)]
mod tests {
    use super::Decrement;
    use crate::{
        command::{request::RequestBuilder, CommandId, Dispatch, DispatchError, Response},
        state::Value,
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_decrement() {
        let mut builder = RequestBuilder::new(CommandId::Decrement);
        assert!(builder.bytes(b"foo".as_ref()).is_ok());
        let req = builder.into_request();
        let hop = Hop::new();
        let mut resp = Vec::new();

        assert!(Decrement::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(Response::from(-1i64).as_bytes(), resp);
        assert_eq!(
            Some(&-1),
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
            Decrement::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }
}
