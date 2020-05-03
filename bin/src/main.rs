#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};
use hop::{command::request::Context, Hop};
use log::{debug, warn};
use std::{error::Error, net::SocketAddr, str::FromStr as _};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    task::block_on(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    debug!("Binding socket");
    let addr = SocketAddr::from_str("127.0.0.1:14000")?;

    debug!("Making TCP listener");
    let listener = TcpListener::bind(&addr).await?;

    let hop = Hop::new();

    let mut incoming = listener.incoming();

    debug!("Listening");

    while let Some(Ok(socket)) = incoming.next().await {
        task::spawn(handle_socket(socket, hop.clone()));
    }

    Ok(())
}

async fn handle_socket(socket: TcpStream, hop: Hop) {
    let addr = socket.peer_addr().unwrap();

    log::debug!("Connected to peer {}", addr);

    match handle_socket_inner(socket, hop).await {
        Ok(()) => debug!("Dropping {}", addr),
        Err(why) => warn!("Erroring {}: {:?}", addr, why),
    }
}

async fn handle_socket_inner(socket: TcpStream, hop: Hop) -> Result<(), Box<dyn Error>> {
    let mut input = Vec::new();
    let mut ctx = Context::new();

    let mut writer = socket.clone();
    let mut reader = BufReader::new(socket);

    while let Ok(size) = reader.read_until(b'\n', &mut input).await {
        // If we get no bytes then we're EOF.
        if size == 0 {
            debug!("Peer no longer sending data");

            break;
        }

        let req = match ctx.feed(&input) {
            Ok(Some(cmd)) => cmd,
            Ok(None) => continue,
            Err(why) => {
                warn!("Failed to feed to context: {:?}", why);

                break;
            }
        };

        let resp = hop.dispatch(&req).unwrap();

        writer.write_all(resp.bytes()).await?;

        if let Some(args) = req.into_args() {
            ctx.reset(args);
        }

        input.clear();
    }

    Ok(())
}
