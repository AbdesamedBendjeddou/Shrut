use crate::components::atoms::{
    messages::{AppMessage, ClientMessage, ServerMessage},
    other_peers_state::OtherPeers,
    this_peer_state::ThisPeer,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use gloo::{
    console::log,
    events::EventListener,
    net::websocket::{futures::WebSocket, Message},
    utils::window,
};
use tokio::sync::broadcast::Sender;
use yew::{platform::spawn_local, UseReducerDispatcher};
use yewdux::prelude::Dispatch;
use AppMessage::*;

pub fn connection_manager(
    other_peers: UseReducerDispatcher<OtherPeers>,
    this_peer: Dispatch<ThisPeer>,
    tx: Sender<AppMessage>,
) {
    let ws = WebSocket::open("ws://127.0.0.1:5050/ws").unwrap();
    let (sender, receiver) = futures::StreamExt::split(ws);
    dispatcher(receiver, tx.clone(), other_peers, this_peer);
    send(sender, tx.clone());
    create_event_send_disconnect_onclose(tx.clone());
}

fn dispatcher(
    mut receiver: SplitStream<WebSocket>,
    tx: Sender<AppMessage>,
    other_peers: UseReducerDispatcher<OtherPeers>,
    this_peer: Dispatch<ThisPeer>,
) {
    spawn_local(async move {
        while let Some(Ok(Message::Text(msg))) = receiver.next().await {
            log!("from recv".to_owned() + &msg);
            let msg = serde_json::from_str::<ServerMessage>(&msg)
                .expect("dispatcher: uncorrect msg format"); // make loop continue if error
            match msg {
                ServerMessage::PeerData(this_peer_data) => {
                    this_peer.reduce(|_| this_peer_data.into())
                }
                ServerMessage::CheckOnline => {
                    tx.send(CltMsg(ClientMessage::CheckOnline))
                        .expect("error sender");
                }
                ServerMessage::SignalingMessage(_) => {
                    tx.send(SrvrMsg(msg)).expect("error sender");
                }
                _ => other_peers.dispatch(msg),
            }
        }
    });
}

fn send(mut sender: SplitSink<WebSocket, Message>, tx: Sender<AppMessage>) {
    let mut rx = tx.subscribe();
    spawn_local(async move {
        while let Ok(msg) = rx.recv().await {
            if let CltMsg(msg) = msg {
                let msg = serde_json::to_string(&msg).unwrap();
                log!(&("from sender".to_owned() + &msg.clone()));
                sender
                    .send(Message::Text(msg))
                    .await
                    .expect("error sending to server") // make loop continue
            }
        }
    });
}

fn create_event_send_disconnect_onclose(tx: Sender<AppMessage>) {
    EventListener::new(&window(), "beforeunload", move |_| {
        tx.send(CltMsg(ClientMessage::Disconnect))
            .expect("error sending from disconnect");
    })
    .forget();
}
