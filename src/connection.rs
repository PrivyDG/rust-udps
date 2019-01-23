use std::sync::*;
use std::sync::atomic::*;
use std::collections::{
    HashSet,
    VecDeque
};
use std::iter::IntoIterator;
use std::ops::DerefMut;

use openssl::rsa::*;
use openssl::pkey::*;

use crate::prelude::*;

pub type ConnectionArc = Arc<Connection>;

/**
 * Struct for storing connection state
*/
pub struct Connection {
    pub id: u32,
    pub address: String,
    pub public_key: RwLock<Option<Rsa<Public>>>,
    pub secret_key_enc: RwLock<Option<Vec<u8>>>,
    pub secret_key_dec: RwLock<Option<Vec<u8>>>,
    pub ping: AtomicU32,
    pub state: RwLock<ConnectionState>,
    pub crypt_state: RwLock<CryptState>,
    pub package_journal: RwLock<HashSet<u32>>,
    pub package_list: RwLock<VecDeque<Package>>,
}

/**
 * enum for storing state of individual connections
*/
pub enum ConnectionState {
    Disconnected = 0,
    Connected,
}

pub enum CryptState {
    None = 0,
    Asymm,
    Symm,
}

impl Connection {
    pub fn new(addr: &String, connection_id: &u32) -> Self {
        Connection {
            id: connection_id.clone(),
            address: addr.clone(),
            ping: AtomicU32::new(0),
            public_key: RwLock::new(None),
            secret_key_enc: RwLock::new(None),
            secret_key_dec: RwLock::new(None),
            state: RwLock::new(
                ConnectionState::Disconnected
            ),
            crypt_state: RwLock::new(
                CryptState::None
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
        let exists = {
            let journal = self.package_journal.read().unwrap();
            journal.contains(&package.header.package_id)
        };
        if !exists {
            let mut package_id: u32 = 0;
            let mut packages = self.package_list.write().unwrap();
            package_id = package.header.package_id;
            packages.push_back(package);
            let mut journal = self.package_journal.write().unwrap();
            journal.insert(package_id);
        }
    }

    pub fn set_pubkey(&self, pubkey_der: Vec<u8>) -> Result<(), String> {
        let rsa_pubkey_res = Rsa::public_key_from_der(pubkey_der.as_slice());
        if rsa_pubkey_res.is_err() {
            return Err("Error decoding DER to public key!".to_string());
        }
        let mut public_key = self.public_key.write().unwrap();
        *public_key = Some(rsa_pubkey_res.unwrap());
        Ok(()) 
    }

    pub fn set_enc_secret(&self, secret_key: Vec<u8>) {
        let mut enc_secret = self.secret_key_enc.write().unwrap();
        *enc_secret = Some(secret_key);
    }

    pub fn set_dec_secret(&self, secret_key: Vec<u8>) {
        let mut dec_secret = self.secret_key_dec.write().unwrap();
        *dec_secret = Some(secret_key);
    }

    pub fn collect_packages(&self) -> Vec<Package> {
        let ret = {
            let packages = self.package_list.read().unwrap();
            packages.iter().map(|p| {
                p.clone()
            }).collect()
        };
        let mut packages = self.package_list.write().unwrap();
        let mut journal = self.package_journal.write().unwrap();
        packages.clear();
        journal.clear();
        ret
    }
}
