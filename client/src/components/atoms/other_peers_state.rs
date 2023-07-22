use yew::Reducible;


use super::{avatar::OtherPeer, messages::ServerMessage};

#[derive(Default, Clone, Debug, PartialEq)]
pub enum WebRTCRole {
    #[default]
    Server,
    Client,
}

#[derive(Default, Clone, PartialEq)]
pub struct OtherPeers {
    pub peers: Vec<OtherPeer>,
}

impl Reducible for OtherPeers {
    type Action = ServerMessage;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            ServerMessage::ConnectedPeers(peers) => { Self { peers } }.into(),
            ServerMessage::PeerJoined(mut peer) => {
                peer.set_role(WebRTCRole::Client);
                let mut peers = self.peers.clone();
                peers.push(peer);
                Self { peers }
            }
            .into(),
            ServerMessage::PeerLeft(id_left) => {
                let mut peers = self.peers.clone();
                peers.retain(|peer| peer.id != id_left);
                Self { peers }
            }
            .into(),
            _ => self.into(),
        }
    }
}
