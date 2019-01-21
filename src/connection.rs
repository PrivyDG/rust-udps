use std::sync::*;
use std::sync::atomic::*;
use std::collections::{
    HashSet,
    VecDeque
};

use crate::prelude::*;

pub type ConnectionArc = Arc<Connection>;

/**
 * Struct for storing connection state
*/
pub struct Connection {
    pub id: u32,
    pub address: String,
    pub ping: AtomicU32,
    pub state: RwLock<ConnectionState>,
    pub package_journal: RwLock<HashSet<u32>>,
    pub package_list: RwLock<VecDeque<Package>>,
}

/**
 * enum for storing state of individual connections
*/
pub enum ConnectionState {
    Disconnected,
    Connected,
}

impl Iterator for Connection {
    type Item = Package;

    fn next(&mut self) -> Option<Package> {
        let mut packages = self.package_list.write().unwrap();
        packages.pop_front()
    }
}

impl Connection {
    pub fn new(addr: &String, connection_id: &u32) -> Self {
        Connection {
            id: connection_id.clone(),
            address: addr.clone(),
            ping: AtomicU32::new(0),
            state: RwLock::new(
                ConnectionState::Disconnected
            ),
            package_journal: RwLock::new(
                HashSet::new()
            ),
            package_list: RwLock::new(
                VecDeque::new()
            )
        }
    }

    pub fn push_package(&self, package: Package) {
        let exists;
        {
            let mut journal = self.package_journal.read().unwrap();
            exists = journal.contains(&package.header.package_id);
        }
        if !exists {
            let mut package_id: u32 = 0;
            {
                let mut packages = self.package_list.write().unwrap();
                package_id = package.header.package_id;
                packages.push_back(package);
            }
            {
                let mut journal = self.package_journal.write().unwrap();
                journal.insert(package_id);
            }
        }
    }
}
