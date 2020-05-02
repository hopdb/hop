mod input;

use async_std::task;
use hop::Client;
use hop_lib::command::CommandId;
use std::{
    error::Error,
    io::{self, BufRead, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stdout = io::stdout();
    let stdout = stdout.lock();

    task::block_on(run(stdin, stdout))
}

async fn run(mut reader: impl BufRead, mut writer: impl Write) -> Result<(), Box<dyn Error>> {
    let mut client = Client::memory();
    let mut input = String::new();

    loop {
        write!(writer, "> ")?;
        writer.flush()?;
        let req = input::process_command(&mut reader, &mut input)?;
        input.clear();

        match req.kind() {
            CommandId::Decrement => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writeln!(writer, "Key required.")?;

                        continue;
                    }
                };

                let v = client.decrement(key).await?;

                writeln!(writer, "{}", v)?;
            }
            CommandId::Echo => {
                if let Some(args) = req.flatten_args() {
                    let v = client.echo(args).await?;

                    writeln!(writer, "{}", String::from_utf8_lossy(&v))?;
                } else {
                    writeln!(writer)?;
                }
            }
            CommandId::Increment => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writeln!(writer, "Key required.")?;

                        continue;
                    }
                };

                let v = client.increment(key).await?;

                writeln!(writer, "{}", v)?;
            }
            _ => {}
        }
    }
}
