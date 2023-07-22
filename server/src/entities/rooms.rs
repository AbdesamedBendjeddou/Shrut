use std::{collections::HashMap, net::IpAddr};

use super::room::Room;




#[derive(Debug)]
pub struct Rooms(HashMap<IpAddr,Room>);


impl Rooms {
    pub fn new()-> Self{
        Rooms(HashMap::new())
    }

    pub fn get(&self, ip: &IpAddr) -> Option<&Room> {
        self.0.get(ip)
    }

    pub fn get_mut(&mut self, ip: &IpAddr) -> Option<&mut Room> {
        self.0.get_mut(ip)
    }

    pub fn get_or_create(&mut self, ip: &IpAddr) -> &mut Room {
        self.0.entry(ip.clone()).or_insert_with(|| Room::new())
    }

    pub fn delete_room(&mut self, ip: &IpAddr) {
        self.0.remove(ip);
    }
}
