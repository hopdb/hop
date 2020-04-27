use super::prelude::*;

pub struct Append;

impl Dispatch for Append {
    fn dispatch(hop: &Hop, req: &mut Request) -> Result<Response> {
        let key = req.arg(0).ok_or(Error::KeyRetrieval)?;
        let arg = req.arg(1).ok_or(Error::ArgumentRetrieval)?;

        let mut value = hop.state().bytes(key).map_err(|_| Error::WrongType)?;
        value.extend_from_slice(arg);

        Ok(Response::from_usize(value.len()))
    }
}
