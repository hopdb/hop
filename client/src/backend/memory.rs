use async_trait::async_trait;
use hop_lib::{
    command::{
        self,
        protocol::CommandInfo,
        CommandError,
        CommandType,
    },
    Hop,
};
use std::convert::TryInto;
use super::Backend;

#[derive(Debug)]
pub enum Error {
    RunningCommand {
        source: CommandError,
    },
}

impl From<CommandError> for Error {
    fn from(source: CommandError) -> Self {
        Self::RunningCommand { source }
    }
}

pub struct MemoryBackend {
    hop: Hop,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            hop: Hop::new(),
        }
    }
}

#[async_trait]
impl Backend for MemoryBackend {
    type Error = Error;

    async fn decrement_int(&mut self, key: &[u8]) -> Result<i64, Self::Error> {
        let cmd = CommandInfo {
            arguments: Some(vec![key.to_vec()]),
            kind: CommandType::DecrementInt,
        };
        let resp = command::dispatch(self.hop.state(), &cmd)?.into_bytes();

        let arr = resp.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    async fn increment(&mut self, key: &[u8]) -> Result<i64, Self::Error> {
        let cmd = CommandInfo {
            arguments: Some(vec![key.to_vec()]),
            kind: CommandType::IncrementInt,
        };
        let resp = command::dispatch(self.hop.state(), &cmd)?.into_bytes();

        let arr = resp.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    async fn increment_int(&mut self, key: &[u8]) -> Result<i64, Self::Error> {
        let cmd = CommandInfo {
            arguments: Some(vec![key.to_vec()]),
            kind: CommandType::IncrementInt,
        };
        let resp = command::dispatch(self.hop.state(), &cmd)?.into_bytes();

        let arr = resp.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }
}
