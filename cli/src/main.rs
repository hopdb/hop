#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

mod input;

use hop::Client;
use hop_engine::command::CommandId;
use std::error::Error;
use tokio::{
    io::{self, AsyncBufRead, AsyncWrite, AsyncWriteExt, BufReader},
    runtime::Builder as RuntimeBuilder,
};

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = BufReader::new(io::stdin());
    let stdout = io::stdout();

    let mut runtime = RuntimeBuilder::new().threaded_scheduler().build()?;

    runtime.block_on(run(stdin, stdout))
}

async fn run(
    mut reader: impl AsyncBufRead + Unpin,
    mut writer: impl AsyncWrite + Unpin,
) -> Result<(), Box<dyn Error>> {
    let client = Client::memory();
    let mut input = String::new();

    loop {
        writer.write_all(b"> ").await?;
        writer.flush().await?;
        let req = input::process_command(&mut reader, &mut input).await?;
        input.clear();

        match req.kind() {
            CommandId::Decrement => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writer.write_all(b"Key required.\n").await?;

                        continue;
                    }
                };

                let v = client.decrement(key).await?;

                writer.write_all(v.to_string().as_bytes()).await?;
                writer.write_all(&[b'\n']).await?;
            }
            CommandId::Echo => {
                if let Some(args) = req.flatten_args() {
                    let v = client.echo(args).await?;

                    writer.write_all(v.as_slice()).await?;
                } else {
                    writer.write_all(&[b'\n']).await?;
                }
            }
            CommandId::Increment => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writer.write_all(b"Key required.\n").await?;

                        continue;
                    }
                };

                let v = client.increment(key).await?;

                writer.write_all(v.to_string().as_bytes()).await?;
                writer.write_all(&[b'\n']).await?;
            }
            CommandId::Stats => {
                let stats = client.stats().await?;

                writer
                    .write_all(
                        format!("Commands successful: {}\n", stats.commands_successful())
                            .as_bytes(),
                    )
                    .await?;
                writer
                    .write_all(
                        format!("Commands errored: {}\n", stats.commands_errored()).as_bytes(),
                    )
                    .await?;
                writer
                    .write_all(
                        format!("Sessions started: {}\n", stats.sessions_started()).as_bytes(),
                    )
                    .await?;
                writer
                    .write_all(format!("Sessions ended: {}\n", stats.sessions_ended()).as_bytes())
                    .await?;
            }
            _ => {}
        }
    }
}
