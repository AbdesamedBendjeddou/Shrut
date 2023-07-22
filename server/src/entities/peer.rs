use axum::http::HeaderMap;
use rnglib::{Language, RNG};
use serde::Serialize;
use std::{net::IpAddr, sync::Arc};
use tokio::sync::broadcast::{error::SendError, Sender};
use user_agent_parser::UserAgentParser;
use uuid::Uuid;

use crate::{
    entities::messages::{AppMessage::*, ServerMessage},
    AppState,
};

use super::{
    messages::{AppMessage, SignalingMessage},
    room::Room,
};

#[derive(Clone, Debug, Serialize)]
pub struct Peer {
    pub id: Uuid,
    pub name: String,
    pub os: String,
    #[serde(skip_serializing)]
    pub ip: IpAddr,
    #[serde(skip_serializing)]
    tx: Sender<AppMessage>,
}

impl Peer {
    pub fn new(headers: HeaderMap, ip: IpAddr, tx: Sender<AppMessage>) -> Self {
        Peer {
            id: Uuid::new_v4(),
            name: Self::generate_name(),
            os: Self::extract_peer_os(headers),
            ip,
            tx,
        }
    }
    pub fn send(&self, message: AppMessage) -> Result<usize, SendError<AppMessage>> {
        self.tx.send(message)?;
        Ok(1)
    }

    pub async fn init(&self, state: Arc<AppState>) {
        let mut rooms = state.rooms.lock().await;
        let room = rooms.get_or_create(&self.ip);
        let other_peers = room.peers();
        self.join_room(room);
        self.send_this_peer_data();
        self.send_peer_joined_to_other_peers(&other_peers);
        self.send_other_peers_data(other_peers);
    }

    pub fn join_room(&self, room: &mut Room) {
        room.receive_peer(self.clone());
    }

    fn send_this_peer_data(&self) {
        let message = SrvrMsg(ServerMessage::PeerData(self.clone()));
        self.send(message).expect("Error sending data");
    }

    fn send_peer_joined_to_other_peers(&self, other_peers: &Vec<Peer>) {
        let message = SrvrMsg(ServerMessage::PeerJoined(self.clone()));
        for peer in other_peers {
            peer.send(message.clone())
                .expect("error sending from send peer joined");
        }
    }

    fn send_other_peers_data(&self, other_peers: Vec<Peer>) {
        let message = SrvrMsg(ServerMessage::ConnectedPeers(other_peers));
        self.send(message)
            .expect("error seding from send other peres data");
    }

    pub fn check_online(&self, state: Arc<AppState>) {}

    pub async fn disconnect(&self, state: Arc<AppState>) {
        println!("disconnect excuted");
        let mut rooms = state.rooms.lock().await;
        {
            let room = rooms.get_mut(&self.ip).unwrap();
            room.remove_peer(&self.id);
            room.peers().iter().for_each(|peer| {
                peer.send(SrvrMsg(ServerMessage::PeerLeft(self.id)))
                    .expect("error sending form disconnect");
            });
        }
        if rooms.get(&self.ip).unwrap().is_empty() {
            rooms.delete_room(&self.ip)
        }
    }

    fn generate_name() -> String {
        let rng = RNG::try_from(&Language::Elven).unwrap();

        let first_name = rng.generate_name();
        let last_name = rng.generate_name();
        format!("{} {}", first_name, last_name)
    }
    pub fn extract_peer_os(headers: HeaderMap) -> String {
        let ua_parser = UserAgentParser::from_path("regexes.yaml").unwrap();
        if let Some(user_agent) = headers.get("user-agent") {
            let os = ua_parser
                .parse_os(user_agent.to_str().unwrap())
                .name
                .unwrap_or("unknown".into())
                .to_string();
            println!("{:?}", os);
            os
        } else {
            "unknown".to_owned()
        }
    }

    pub async fn signal(&self, message: SignalingMessage, state: Arc<AppState>) {
        let (other_peer_id, message) = message.replace_other_id_with_this_peer_id(self.id.clone());

        let rooms = state.rooms.lock().await;
        let room = rooms.get(&self.ip).expect("no room fond");
        let other_peer = room.get(&other_peer_id).expect("no peer with this id");
        other_peer
            .send(SrvrMsg(ServerMessage::SignalingMessage(message)))
            .expect("error sending from signal");
    }
}
