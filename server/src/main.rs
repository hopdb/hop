#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

use hop_engine::{
    command::{request::Context, Response},
    Hop,
};
use log::{debug, warn};
use std::{
    env,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr as _,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    runtime::Builder as RuntimeBuilder,
    stream::StreamExt,
    task,
};

struct Config {
    host: IpAddr,
    port: u16,
}

impl Config {
    const HOST_DEFAULT: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    const PORT_DEFAULT: u16 = 46733;

    fn new() -> Self {
        let host = match env::var("HOST") {
            Ok(host) => IpAddr::from_str(&host).unwrap_or(Self::HOST_DEFAULT),
            Err(_) => Self::HOST_DEFAULT,
        };
        let port = match env::var("PORT") {
            Ok(port) => port.parse().unwrap_or(Self::PORT_DEFAULT),
            Err(_) => Self::PORT_DEFAULT,
        };

        Self { host, port }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut runtime = RuntimeBuilder::new().threaded_scheduler().build()?;

    runtime.block_on(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let config = Config::new();

    debug!("Binding socket");
    let addr = SocketAddr::new(config.host, config.port);

    debug!("Binding to {}", addr);
    let mut listener = TcpListener::bind(&addr).await?;

    let hop = Hop::new();

    let mut incoming = listener.incoming();

    debug!("Listening for new connections on {}", addr);

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

    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut resp = Vec::new();

    while let Ok(size) = reader.read_until(b'\n', &mut input).await {
        // If we get no bytes then we're EOF.
        if size == 0 {
            break;
        }

        match ctx.feed(&input) {
            Ok(Some(req)) => {
                match hop.dispatch(&req, &mut resp) {
                    Ok(()) => {}
                    Err(why) => {
                        let res = Response::DispatchError(why);

                        res.copy_to(&mut resp);
                    }
                }

                if let Some(args) = req.into_args() {
                    ctx.reset(args);
                }
            }
            Ok(None) => continue,
            Err(why) => {
                let res = Response::ParseError(why);

                res.copy_to(&mut resp);
            }
        };

        writer.write_all(&resp).await?;
        resp.clear();
        input.clear();
    }

    Ok(())
}
