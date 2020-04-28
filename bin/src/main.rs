use futures_util::{
    io::{BufReader, AsyncBufReadExt, AsyncWriteExt},
    stream::StreamExt,
};
use hop::{
    command::protocol::Context,
    Hop,
};
use log::{debug, warn};
use smol::{Async, Task};
use std::{
    error::Error,
    net::{SocketAddr, TcpListener, TcpStream},
    str::FromStr as _,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    smol::run(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    debug!("Binding socket");
    let addr = SocketAddr::from_str("127.0.0.1:14000")?;

    debug!("Making TCP listener");
    let listener = Async::<TcpListener>::bind(&addr)?;

    let hop = Hop::new();

    let mut incoming = listener.incoming();

    debug!("Listening");

    while let Some(Ok(socket)) = incoming.next().await {
        Task::spawn(handle_socket(socket, hop.clone())).detach();
    }

    Ok(())
}

async fn handle_socket(socket: Async<TcpStream>, hop: Hop) {
    let addr = socket.get_ref().peer_addr().unwrap();

    log::debug!("Connected to peer {}", addr);

    match handle_socket_inner(socket, hop).await {
        Ok(()) => debug!("Dropping {}", addr),
        Err(why) => warn!("Erroring {}: {:?}", addr, why),
    }
}

async fn handle_socket_inner(socket: Async<TcpStream>, hop: Hop) -> Result<(), Box<dyn Error>> {
    let mut input = Vec::new();
    let mut ctx = Context::new();

    let writer = socket.get_ref().try_clone()?;
    let mut writer = Async::new(writer)?;
    let mut reader = BufReader::new(socket);

    while let Ok(size) = reader.read_until(b'\n', &mut input).await {
        // If we get no bytes then we're EOF.
        if size == 0 {
            debug!("Peer no longer sending data");

            break;
        }

        let mut req = match ctx.feed(&input) {
            Ok(Some(cmd)) => cmd,
            Ok(None) => continue,
            Err(why) => {
                warn!("Failed to feed to context: {:?}", why);

                break;
            }
        };

        let resp = hop.dispatch(&mut req).unwrap();

        writer.write_all(resp.bytes()).await?;

        if let Some(args) = req.into_args() {
            ctx.reset(args);
        }

        input.clear();
    }

    Ok(())
}
