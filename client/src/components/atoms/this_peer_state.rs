use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yewdux::store::Store;


#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Store, Debug)]
#[store(storage = "session")]
pub struct ThisPeer {
    pub name: String,
    pub id: Uuid,
  
}
