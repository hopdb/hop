use async_std::task;
use hop::Client;
use std::{
    env,
    error::Error,
    io::{self, StdinLock, StdoutLock, Write},
    net::SocketAddr,
    str::FromStr,
};

fn main() -> Result<(), Box<dyn Error>> {
    let addr = address().unwrap();

    let client = task::block_on(Client::connect(addr))?;
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;
        let cmd = command(&mut stdin);
    }
}

fn address() -> Option<SocketAddr> {
    let arg = env::args().skip(1).next()?;

    SocketAddr::from_str(&arg).ok()
}

fn command(lock: &mut StdinLock) -> String {
    let mut s = String::new();
    s
}
