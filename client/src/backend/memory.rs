use super::Backend;
use crate::model::StatsData;
use async_trait::async_trait;
use hop_engine::{
    command::{
        response::{Context, Instruction, Response},
        CommandId, DispatchError, Request,
    },
    state::{KeyType, Value},
    Hop,
};
use std::{
    convert::TryInto,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

fn request(id: CommandId, args: Option<Vec<Vec<u8>>>, kind: Option<KeyType>) -> Request {
    if let Some(kind) = kind {
        Request::new_with_type(id, args, kind)
    } else {
        Request::new(id, args)
    }
}

#[derive(Debug)]
pub enum Error {
    RunningCommand { source: DispatchError },
}

impl From<DispatchError> for Error {
    fn from(source: DispatchError) -> Self {
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

    async fn decrement(&self, key: &[u8], kind: Option<KeyType>) -> Result<i64, Self::Error> {
        let req = request(CommandId::Decrement, Some(vec![key.to_vec()]), kind);
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let arr = resp.get(1..9).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    async fn echo(&self, content: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let req = Request::new(CommandId::Echo, Some(vec![content.to_vec()]));
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        if !resp.is_empty() {
            resp.remove(resp.len() - 1);
        }

        Ok(resp)
    }

    async fn increment(&self, key: &[u8], kind: Option<KeyType>) -> Result<i64, Self::Error> {
        let req = request(CommandId::Increment, Some(vec![key.to_vec()]), kind);
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let arr = resp.get(1..9).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    async fn rename(&self, from: &[u8], to: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let req = Request::new(CommandId::Rename, Some(vec![from.to_vec(), to.to_vec()]));
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        let bytes = match resp {
            Response::Value(Value::Bytes(bytes)) => bytes,
            _ => panic!(),
        };

        Ok(bytes)
    }

    async fn stats(&self) -> Result<StatsData, Self::Error> {
        let req = request(CommandId::Stats, None, None);
        let mut resp = Vec::new();

        self.hop.dispatch(&req, &mut resp)?;

        let mut ctx = Context::new();

        let resp = match ctx.feed(&resp).unwrap() {
            Instruction::Concluded(value) => value,
            Instruction::ReadBytes(_) => unreachable!(),
        };

        let stats = match resp {
            Response::Value(Value::Map(stats)) => stats,
            _ => panic!(),
        };

        Ok(StatsData::new(stats.into_iter().collect()))
    }
}
