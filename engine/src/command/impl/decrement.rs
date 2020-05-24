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
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::object::Integer,
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_decrement() {
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let req = Request::new(CommandId::Decrement, Some(args));
        let hop = Hop::new();
        let mut resp = Vec::new();

        assert!(Decrement::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(Response::from(-1i64).as_bytes(), resp);
        assert_eq!(
            Some(-1),
            hop.state().typed_key::<Integer>(b"foo").as_deref().copied()
        );
    }

    #[test]
    fn test_no_key() {
        let req = Request::new(CommandId::Decrement, None);
        let hop = Hop::new();
        let mut resp = Vec::new();

        assert_eq!(
            DispatchError::KeyUnspecified,
            Decrement::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }
}
