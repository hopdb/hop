use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};
use crate::error::Result;
use futures::StreamExt;
use imms::{command::{self, protocol::Context}, Imms};
use log::{debug, warn};
use std::{
    net::SocketAddr,
    str::FromStr as _,
};

async fn handle_socket(socket: TcpStream, state: Imms) {
    let addr = socket.peer_addr().unwrap();

    log::debug!("Connected to peer {}", addr);

    match handle_socket_inner(socket, state).await {
        Ok(()) => debug!("Dropping {}", addr),
        Err(why) => warn!("Erroring {}: {:?}", addr, why),
    }
}

async fn handle_socket_inner(socket: TcpStream, mut state: Imms) -> Result<()> {
    let mut input = Vec::new();
    let mut ctx = Context::new();

    let mut writer = socket.clone();
    let mut reader = BufReader::new(socket);

    while let Ok(size) = reader.read_until(b'\n', &mut input).await {
        // If we get no bytes then that means we're EOF.
        if size == 0 {
            debug!("Peer no longer sending data");

            break;
        }

        debug!("bytes: {:?}", input);

        let res = ctx.feed(&mut input);

        let cmd = match res {
            Ok(Some(cmd)) => cmd,
            Ok(None) => continue,
            Err(why) => {
                warn!("Failed to feed to context: {:?}", why);

                break;
            },
        };

        let resp = command::dispatch(&mut state, &cmd).unwrap();

        debug!("writing");
        writer.write_all(resp.bytes()).await?;
        debug!("written");

        if let Some(arguments) = cmd.arguments {
            ctx.reset(arguments);
        }
        input.clear();
    }

    Ok(())
}

pub async fn run() -> Result<()> {
    debug!("Binding socket");
    let addr = SocketAddr::from_str("127.0.0.1:14000")?;
    debug!("Bound");

    debug!("Making listener");
    let listener = TcpListener::bind(&addr).await?;

    debug!("Initializing state");
    let state = Imms::new();

    let mut incoming = listener.incoming();

    debug!("Listening");

    while let Some(Ok(socket)) = incoming.next().await {
        task::spawn(handle_socket(socket, state.clone()));
    }

    Ok(())
}
