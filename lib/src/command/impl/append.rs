use super::prelude::*;
use crate::state::object::List;

pub struct Append;

impl Dispatch for Append {
    fn dispatch(hop: &Hop, req: &mut Request) -> Result<Response> {
        let key = req.arg(0).ok_or(Error::KeyRetrieval)?;
        let arg = req.arg(1).ok_or(Error::ArgumentRetrieval)?;

        let mut value = hop.state().typed_key::<List>(key).ok_or(Error::WrongType)?;
        value.push(arg.to_vec());

        Ok(Response::from_usize(value.len()))
    }
}
