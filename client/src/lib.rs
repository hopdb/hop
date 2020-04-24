use async_std::{
    io::BufReader,
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
};
use std::{
    convert::TryInto,
    io::Result,
};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn connect(addrs: impl ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(addrs).await?;

        Ok(Self {
            stream,
        })
    }

    pub async fn decrement_int(&mut self, key: impl AsRef<str>) -> Result<i64> {
        let mut cmd = vec![1, 1, 0, 0, 0, key.as_ref().as_bytes().len() as u8];
        cmd.extend_from_slice(key.as_ref().as_bytes());
        cmd.push(b'\n');

        self.stream.write_all(&cmd).await?;

        let mut s = Vec::new();
        let mut reader = BufReader::new(&self.stream);
        reader.read_until(b'\n', &mut s).await?;

        let arr = s.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    pub async fn increment(&mut self, key: impl AsRef<str>) -> Result<i64> {
        let mut cmd = vec![0, 1, 0, 0, 0, key.as_ref().as_bytes().len() as u8];
        cmd.extend_from_slice(key.as_ref().as_bytes());
        cmd.push(b'\n');

        self.stream.write_all(&cmd).await?;

        let mut s = Vec::new();
        let mut reader = BufReader::new(&self.stream);
        reader.read_until(b'\n', &mut s).await?;

        let arr = s.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }

    pub async fn increment_int(&mut self, key: impl AsRef<str>) -> Result<i64> {
        let mut cmd = vec![0, 1, 0, 0, 0, key.as_ref().as_bytes().len() as u8];
        cmd.extend_from_slice(key.as_ref().as_bytes());
        cmd.push(b'\n');

        self.stream.write_all(&cmd).await?;

        let mut s = Vec::new();
        let mut reader = BufReader::new(&self.stream);
        reader.read_until(b'\n', &mut s).await?;

        let arr = s.get(..8).unwrap().try_into().unwrap();
        let num = i64::from_be_bytes(arr);

        Ok(num)
    }
}
