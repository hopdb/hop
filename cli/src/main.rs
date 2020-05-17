#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

mod input;

use hop::{backend::memory::Error as MemoryError, Client};
use hop_engine::command::{CommandId, DispatchError};
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
            CommandId::Delete => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writer
                            .write_all(b"The key to delete is required.\n")
                            .await?;

                        continue;
                    }
                };

                let v = match client.delete(key).await {
                    Ok(v) => v,
                    Err(MemoryError::RunningCommand { source }) => {
                        writer.write_all(b"Dispatch error: ").await?;

                        match source {
                            DispatchError::PreconditionFailed => {
                                let key = String::from_utf8_lossy(key);

                                writer
                                    .write_all(
                                        format!("the key \"{}\" doesn't exist.\n", key).as_bytes(),
                                    )
                                    .await?;
                            }
                            _ => unreachable!(),
                        }

                        continue;
                    }
                };

                writer
                    .write_all(String::from_utf8_lossy(&v).as_bytes())
                    .await?;
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
            CommandId::Rename => {
                let from = match req.key() {
                    Some(key) => key,
                    None => {
                        writer
                            .write_all(b"The key to rename is required.\n")
                            .await?;

                        continue;
                    }
                };
                let to = match req.arg(1) {
                    Some(to) => to,
                    None => {
                        writer
                            .write_all(b"The destination key name is required.\n")
                            .await?;

                        continue;
                    }
                };

                let v = match client.rename(from, to).await {
                    Ok(v) => v,
                    Err(MemoryError::RunningCommand { source }) => {
                        writer.write_all(b"Dispatch error: ").await?;

                        match source {
                            DispatchError::KeyRetrieval => {
                                let from = String::from_utf8_lossy(from);

                                writer
                                    .write_all(
                                        format!("key \"{}\" doesn't exist\n", from).as_bytes(),
                                    )
                                    .await?;
                            }
                            DispatchError::PreconditionFailed => {
                                let to = String::from_utf8_lossy(to);

                                writer
                                    .write_all(
                                        format!(
                                            "the destination key, \"{}\", already exists.\n",
                                            to
                                        )
                                        .as_bytes(),
                                    )
                                    .await?;
                            }
                            _ => unreachable!(),
                        }

                        continue;
                    }
                };

                writer
                    .write_all(String::from_utf8_lossy(&v).as_bytes())
                    .await?;
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
