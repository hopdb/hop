#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

mod input;

use async_std::{
    io::{self, prelude::*, BufReader},
    task,
};
use hop::Client;
use hop_lib::command::CommandId;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let stdin = BufReader::new(task::block_on(stdin.lock()));
    let stdout = io::stdout();
    let stdout = task::block_on(stdout.lock());

    task::block_on(run(stdin, stdout))
}

async fn run(
    mut reader: impl BufRead + Unpin,
    mut writer: impl Write + Unpin,
) -> Result<(), Box<dyn Error>> {
    let client = Client::memory();
    let mut input = String::new();

    loop {
        write!(writer, "> ").await?;
        writer.flush().await?;
        let req = input::process_command(&mut reader, &mut input).await?;
        input.clear();

        match req.kind() {
            CommandId::Decrement => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writeln!(writer, "Key required.").await?;

                        continue;
                    }
                };

                let v = client.decrement(key).await?;

                writeln!(writer, "{}", v).await?;
            }
            CommandId::Echo => {
                if let Some(args) = req.flatten_args() {
                    let v = client.echo(args).await?;

                    writeln!(writer, "{}", String::from_utf8_lossy(&v)).await?;
                } else {
                    writeln!(writer).await?;
                }
            }
            CommandId::Increment => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writeln!(writer, "Key required.").await?;

                        continue;
                    }
                };

                let v = client.increment(key).await?;

                writeln!(writer, "{}", v).await?;
            }
            _ => {}
        }
    }
}
