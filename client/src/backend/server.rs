use async_std::{
    io::BufReader,
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
};
use async_trait::async_trait;
use std::{
    convert::TryInto,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    result::Result as StdResult,
};
use super::Backend;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Connecting {
        source: IoError,
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Connecting { .. } => f.write_str("failed to connect"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Connecting { source } => Some(source),
        }
    }
}

pub struct ServerBackend {
    stream: TcpStream,
}

impl ServerBackend {
    pub async fn connect(addrs: impl ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(addrs).await.map_err(|source| Error::Connecting { source })?;

        Ok(Self { stream })
    }

}

#[async_trait]
impl Backend for ServerBackend {
    type Error = Error;

    async fn decrement_int(&mut self, key: &[u8]) -> Result<i64> {
        let mut cmd = vec![1, 1, 0, 0, 0, key.len() as u8];
        cmd.extend_from_slice(key);
        cmd.push(b'\n');

        self.stream.write_all(&cmd).await.unwrap();

        let mut s = Vec::new();
        let mut reader = BufReader::new(&self.stream);
        reader.read_until(b'\n', &mut s).await.unwrap();

        let arr = s.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    async fn increment(&mut self, key: &[u8]) -> Result<i64> {
        let mut cmd = vec![0, 1, 0, 0, 0, key.len() as u8];
        cmd.extend_from_slice(key);
        cmd.push(b'\n');

        self.stream.write_all(&cmd).await.unwrap();

        let mut s = Vec::new();
        let mut reader = BufReader::new(&self.stream);
        reader.read_until(b'\n', &mut s).await.unwrap();

        let arr = s.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    async fn increment_int(&mut self, key: &[u8]) -> Result<i64> {
        let mut cmd = vec![0, 1, 0, 0, 0, key.len() as u8];
        cmd.extend_from_slice(key);
        cmd.push(b'\n');

        self.stream.write_all(&cmd).await.unwrap();

        let mut s = Vec::new();
        let mut reader = BufReader::new(&self.stream);
        reader.read_until(b'\n', &mut s).await.unwrap();

        let arr = s.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }
}
