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

async fn run(mut stdin: impl BufRead, mut stdout: impl Write) -> Result<(), Box<dyn Error>> {
    let mut client = Client::memory();
    let mut input = String::new();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;
        let req = input::process_command(&mut stdin, &mut input)?;
        input.clear();

        match req.kind() {
            CommandId::Decrement => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writeln!(stdout, "Key required.")?;

                        continue;
                    }
                };

                let v = client.decrement(key).await?;

                writeln!(stdout, "{}", v)?;
            }
            CommandId::Echo => {
                if let Some(args) = req.flatten_args() {
                    let v = client.echo(args).await?;

                    writeln!(stdout, "{}", String::from_utf8_lossy(&v))?;
                } else {
                    writeln!(stdout)?;
                }
            }
            CommandId::Increment => {
                let key = match req.key() {
                    Some(key) => key,
                    None => {
                        writeln!(stdout, "Key required.")?;

                        continue;
                    }
                };

                let v = client.increment(key).await?;

                writeln!(stdout, "{}", v)?;
            }
            _ => {}
        }
    }
}
