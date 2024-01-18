use std::{sync::Arc, net::{SocketAddr, IpAddr, Ipv6Addr}, str::FromStr};

use axum::{Router, routing::get};
use clap::Parser;
use entities::rooms::Rooms;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

use crate::connection_manager::socket_handler;

mod entities;
mod connection_manager;

#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,
    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "0.0.0.0")]
    addr: String,
    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "5050")]
    port: u16,
}

#[derive(Debug)]
pub struct AppState {
    rooms: Mutex<Rooms>,
}

pub async fn run() {
    let opt = Opt::parse();
    let state = Arc::new(AppState {
        rooms: Mutex::new(Rooms::new()),
    });

    let app = Router::new()
        .route("/ws", get(socket_handler))
        .layer(ServiceBuilder::new())
        .with_state(state)
        .nest_service("/", ServeDir::new("../dist"));

    let addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    println!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Unable to start server");
}
