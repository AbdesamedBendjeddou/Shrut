use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::peer::Peer;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SignalingMessage {
    Offer(Uuid, String),
    Answer(Uuid, String),
    IceCandidate(Uuid, IceCandidate),
}

impl SignalingMessage {
  pub  fn replace_other_id_with_this_peer_id(self, this_peer: Uuid) -> (Uuid, SignalingMessage) {
        match self {
            SignalingMessage::Offer(other_peer, offer) => (other_peer,SignalingMessage::Offer(this_peer, offer)),
            SignalingMessage::Answer(other_peer, answer) => (other_peer,SignalingMessage::Answer(this_peer, answer)),
            SignalingMessage::IceCandidate(other_peer, ice_candidate) => (other_peer,SignalingMessage::IceCandidate(this_peer, ice_candidate)),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub enum ClientMessage {
    CheckOnline,
    Disconnect,
    SignalingMessage(SignalingMessage),
}

#[derive(Serialize, Debug, Clone)]
pub enum ServerMessage {
    PeerData(Peer),           
    PeerJoined(Peer),          
    ConnectedPeers(Vec<Peer>), 
    PeerLeft(Uuid),           
    CheckOnline,
    SignalingMessage(SignalingMessage),
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    SrvrMsg(ServerMessage),
    CltMsg(ClientMessage),
}
