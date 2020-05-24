use super::{
    super::{Dispatch, DispatchError, DispatchResult, Request},
    increment_by::IncrementBy,
};
use crate::Hop;
use alloc::vec::Vec;

pub struct Increment;

impl Dispatch for Increment {
    fn dispatch(hop: &Hop, req: &Request, resp: &mut Vec<u8>) -> DispatchResult<()> {
        let key = req.key().ok_or(DispatchError::KeyUnspecified)?;

        IncrementBy::increment(hop, key, resp)
    }
}

#[cfg(test)]
mod tests {
    use super::Increment;
    use crate::{
        command::{CommandId, Dispatch, DispatchError, Request, Response},
        state::object::Integer,
        Hop,
    };
    use alloc::vec::Vec;

    #[test]
    fn test_increment() {
        let mut args = Vec::new();
        args.push(b"foo".to_vec());
        let req = Request::new(CommandId::Increment, Some(args));
        let hop = Hop::new();
        let mut resp = Vec::new();

        assert!(Increment::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(Response::from(1i64).as_bytes(), resp);
        assert_eq!(
            Some(1),
            hop.state().typed_key::<Integer>(b"foo").as_deref().copied()
        );
    }

    #[test]
    fn test_no_key() {
        let req = Request::new(CommandId::Increment, None);
        let hop = Hop::new();
        let mut resp = Vec::new();

        assert_eq!(
            DispatchError::KeyUnspecified,
            Increment::dispatch(&hop, &req, &mut resp).unwrap_err()
        );
    }
}
