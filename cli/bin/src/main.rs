#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

use hop::Client;
use std::error::Error;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    runtime::Builder as RuntimeBuilder,
};

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = BufReader::new(io::stdin());
    let stdout = io::stdout();

    let mut runtime = RuntimeBuilder::new().basic_scheduler().build()?;

    runtime.block_on(run(stdin, stdout))
}

async fn run(
    mut reader: impl AsyncBufReadExt + Unpin,
    mut writer: impl AsyncWrite + Unpin,
) -> Result<(), Box<dyn Error>> {
    let client = Client::memory();
    let mut input = String::new();

    loop {
        writer.write_all(b"> ").await?;
        writer.flush().await?;
        reader.read_line(&mut input).await?;

        let mut output = hop_cli::process(&client, input.trim_end()).await?;
        output.to_mut().push('\n');

        writer.write_all(output.as_bytes()).await?;
        input.clear();
    }
}
