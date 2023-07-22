use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    http::HeaderMap,
    response::IntoResponse,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast::{channel, Sender};

use crate::{
    entities::{
        messages::{
            AppMessage::{self, *},
            ClientMessage,
        },
        peer::Peer,
    },
    AppState,
};

pub async fn socket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    ws.on_upgrade(move |ws| connection_manager(ws, state, socket_addr, headers))
}

pub async fn connection_manager(
    ws: WebSocket,
    state: Arc<AppState>,
    socket_addr: SocketAddr,
    headers: HeaderMap,
) {
    let (sender, receiver) = ws.split();
    let (tx, _) = channel(100);

    let mut tasks = vec![];

    tasks.push(tokio::spawn(listen(receiver, tx.clone())));
    tasks.push(tokio::spawn(send(sender, tx.clone())));
    tasks.push(tokio::spawn(dispatch(socket_addr, headers, state, tx)));

    for task in tasks {
        task.await.expect("error joining tasks");
    }
}

async fn dispatch(
    socket_addr: SocketAddr,
    headers: HeaderMap,
    state: Arc<AppState>,
    tx: Sender<AppMessage>,
) {
    let this_peer_ip = socket_addr.ip();
    let this_peer = Peer::new(headers, this_peer_ip, tx.clone());
    this_peer.init(state.clone()).await;
    let mut rx = tx.subscribe();
    while let Ok(message) = rx.recv().await {
        if let CltMsg(message) = message {
            match message {
                ClientMessage::CheckOnline => this_peer.check_online(state.clone()),
                ClientMessage::Disconnect => this_peer.disconnect(state.clone()).await,
                ClientMessage::SignalingMessage(message) => {
                    this_peer.signal(message, state.clone()).await
                }
            }
        }
    }
}

async fn listen(mut receiver: SplitStream<WebSocket>, tx: Sender<AppMessage>) {
    while let Some(Ok(Message::Text(message))) = receiver.next().await {
        println!("from listner: {}", message);
        let peer_message = serde_json::from_str::<ClientMessage>(&message)
            .expect("client message is not a valid peer message");
        tx.send(CltMsg(peer_message))
            .expect("error sending data from listner");
    }
}

async fn send(mut sender: SplitSink<WebSocket, Message>, tx: Sender<AppMessage>) {
    let mut rx = tx.subscribe();
    while let Ok(message) = rx.recv().await {
        if let SrvrMsg(message) = message {
            let message = serde_json::to_string(&message).unwrap();
            println!("from sender: {}", message);
            sender
                .send(Message::Text(message))
                .await
                .expect("error sending to server") // make loop continue
        }
    }
    print!("rx dropped")
}
