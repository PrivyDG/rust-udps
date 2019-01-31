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
 * # Struct for storing connection state
 */
pub struct Connection {
    /**
     * Unique connection id
     */
    pub id: u32,
    /**
     * Address to respond to
     */
    pub address: String,
    /**
     * Public key to encode decode key with
     */
    pub public_key: RwLock<Option<Rsa<Public>>>,
    /**
     * Secret key to decode data with
     */
    pub secret_key: RwLock<Option<Vec<u8>>>,
    /**
     * Self-explanatory
     */
    pub ping: AtomicU32,
    /**
     * Current connection state
     */
    pub state: RwLock<ConnectionState>,
    /**
     * Current possible/enabled level of encryption
     */
    pub crypt_state: RwLock<CryptState>,
    /**
     * Package journal for O(1) lookup of packages in queue
     */
    pub package_journal: RwLock<HashSet<u32>>,
    /**
     * Actual queue of packages
     */
    pub package_list: RwLock<VecDeque<Package>>,
}

/**
 * Enum for storing state of individual connections
 */
#[derive(PartialEq)]
pub enum ConnectionState {
    Disconnected = 0,
    Connected,
}
/**
 * Unused
 */
#[derive(PartialEq)]
pub enum CryptState {
    None = 0,
    Asymm,
    Symm,
}

impl Connection {
    /**
     * Creates a new connection with a given remote address and IP.
     */
    pub fn new(addr: &String, connection_id: &u32) -> Self {
        Connection {
            id: connection_id.clone(),
            address: addr.clone(),
            ping: AtomicU32::new(0),
            public_key: RwLock::new(None),
            secret_key: RwLock::new(None),
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

    /**
     * Pushes a new package to the internal queue, dropping it  
     * if it is a duplicate.
     * This function is thread-safe.
     */
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

    /**
     * Sets the connections public key from binary DER.
     */
    pub fn set_public_key(&self, pubkey_der: Vec<u8>) -> Result<(), String> {
        let rsa_pubkey_res = Rsa::public_key_from_der(pubkey_der.as_slice());
        if rsa_pubkey_res.is_err() {
            return Err("Error decoding DER to public key!".to_string());
        }
        let mut public_key = self.public_key.write().unwrap();
        *public_key = Some(rsa_pubkey_res.unwrap());
        Ok(()) 
    }

    /**
     * Gets the connections public key.
     */
    pub fn get_public_key(&self) -> Option<Rsa<Public>> {
        let pubkey = self.public_key.read().unwrap();
        pubkey.clone()
    }

    /**
     * Sets the connections decoding key.
     */
    pub fn set_secret(&self, secret_key: Vec<u8>) {
        let mut dec_secret = self.secret_key.write().unwrap();
        *dec_secret = Some(secret_key);
    }

    /**
     * Gets the connections decoding key.
     */
    pub fn get_secret(&self) -> Vec<u8> {
        let dec_secret_guard = self.secret_key.read().unwrap();
        let dec_secret = dec_secret_guard.clone();
        dec_secret.unwrap()
    }

    /**
     * Retrieves and collects all packages that accumulated  
     * in the queue since the last call and clears it afterwards.
     */
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
