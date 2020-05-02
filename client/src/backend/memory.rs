use super::Backend;
use async_trait::async_trait;
use hop_lib::{
    command::{CommandError, CommandId, Request},
    Hop,
};
use std::{
    convert::TryInto,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub enum Error {
    RunningCommand { source: CommandError },
}

impl From<CommandError> for Error {
    fn from(source: CommandError) -> Self {
        Self::RunningCommand { source }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::RunningCommand { source } => f.write_fmt(format_args!("{}", source)),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::RunningCommand { .. } => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct MemoryBackend {
    hop: Hop,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
impl Backend for MemoryBackend {
    type Error = Error;

    async fn decrement(&self, key: &[u8]) -> Result<i64, Self::Error> {
        let req = Request::new(CommandId::Decrement, Some(vec![key.to_vec()]));

        let resp = self.hop.dispatch(&req)?.into_bytes();

        let arr = resp.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    async fn echo(&self, content: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let req = Request::new(CommandId::Echo, Some(vec![content.to_vec()]));

        let mut resp = self.hop.dispatch(&req)?.into_bytes();

        if !resp.is_empty() {
            resp.remove(resp.len() - 1);
        }

        Ok(resp)
    }

    async fn increment(&self, key: &[u8]) -> Result<i64, Self::Error> {
        let req = Request::new(CommandId::Increment, Some(vec![key.to_vec()]));
        let resp = self.hop.dispatch(&req)?.into_bytes();

        let arr = resp.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }
}
