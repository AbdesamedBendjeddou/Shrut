use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::webrtc_manager::IceCandidate;

use super::{avatar::OtherPeer, this_peer_state::ThisPeer};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SignalingMessage {
    Offer(Uuid, String),
    Answer(Uuid, String),
    IceCandidate(Uuid, IceCandidate),
}

#[derive(Clone, Serialize, Debug)]
pub enum ClientMessage {
    CheckOnline,
    Disconnect,
    SignalingMessage(SignalingMessage),
}

#[derive(Clone, Deserialize, Debug)]
pub enum ServerMessage {
    PeerJoined(OtherPeer),
    ConnectedPeers(Vec<OtherPeer>),
    PeerLeft(Uuid),
    PeerData(ThisPeer),
    CheckOnline,
    SignalingMessage(SignalingMessage),
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    SrvrMsg(ServerMessage),
    CltMsg(ClientMessage),
}
