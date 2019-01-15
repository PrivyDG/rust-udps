use std::sync::*;
use std::collections::VecDeque;

use crate::prelude::*;

/**
 * Struct for storing connection state
 */
pub struct Connection {
    pub id: u32,
    pub addr: String,
    pub packages: Mutex<VecDeque<Package>>,
}

/**
 * enum for storing state of individual connections
 */
pub enum ConnectionState {
    None = 0,
    Has,
    Lol,
}

impl Iterator for Connection {
    type Item = Package;

    fn next(&mut self) -> Option<Package> {
        let mut packages = self.packages.lock().unwrap();
        if packages.len() < 1 {
            return None;
        }
        packages.pop_back()
    }
}

impl Connection {
    pub fn new(addr: &String, connection_id: &u32) -> Self {
        Connection {
            id: connection_id.clone(),
            addr: addr.clone(),
            packages: Mutex::new(
                VecDeque::new()
            )
        }
    }

    pub fn push_package(&mut self, package: Package) {
        let mut packages = self.packages.lock().unwrap();
        packages.push_back(package);
    }
}