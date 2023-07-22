use std::collections::HashMap;
use uuid::Uuid;

use super::peer::Peer;

#[derive(Debug, Default)]
pub struct Room {
    peers: HashMap<Uuid, Peer>,
}

impl Room {
    pub fn new() -> Self {
        Room {
            peers: HashMap::new(),
        }
    }

    pub fn get(&self, id: &Uuid) -> Option<&Peer> {
        self.peers.get(id)
    }

    pub fn receive_peer(&mut self, peer: Peer) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub fn remove_peer(&mut self, id: &Uuid) {
        self.peers.remove(id);
    }

    pub fn peers(&self) -> Vec<Peer> {
        self.peers.values().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }
}
