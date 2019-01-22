use std::convert::*;
use std::option::*;
use std::vec::*;
use std::mem::*;
use std::time::*;
use std::clone::Clone;

use rand::prelude::*;
use serde::*;
use rmps::*;

use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Header {
    pub version: [u8; 3],
    pub enc_type: EncType,
    pub crypt_type: CryptType,
    pub method_type: MethodType,
    pub connection_id: u32,
    pub package_id: u32,
    pub ack: bool,
    pub sequence_len: Option<u32>,
    pub sequence_ind: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Package {
    pub header: Header,
    pub data: Vec<u8>
}


#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum EncType {
    Raw = 0,
    ZIP,
    LZO    
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum CryptType {
    None = 0,
    Asymm,
    Symm
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum MethodType {
    Connect = 0,
    Disconnect,
    Ack,
    AsymmKey,
    SymmKey,
    Data,
}

impl Package {
    /**
     * Creates a new package with default settings.
     */
    pub fn new_default() -> Self {
        Self {
            header: Header {
                version: crate::VERSION,
                enc_type: EncType::Raw,
                crypt_type: CryptType::None,
                method_type: MethodType::Data,
                connection_id: thread_rng().next_u32(),
                package_id: thread_rng().next_u32(),
                ack: false,
                sequence_ind: None,
                sequence_len: None 
            },
            data: Vec::new()
        }
    }
}

pub struct PackageAck {
    pub cached_package: Package,
    pub timestamp: Instant,
    pub attempts: u8,
}

impl TryFrom<Vec<u8>> for Package {
    type Error = String;

    /**
     * Try decoding from binary (MessagePack encoded)
     */
    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let slice = data.as_slice();
        let package_res = from_slice::<Self>(slice);
        if package_res.is_err() {
            return Err("Unknown error decoding MessagePack package!".to_string());
        }
        Ok(
            package_res.unwrap()
        )
    }
}

impl TryInto<Vec<u8>> for Package {
    type Error = String;

    /**
     * Try encoding into binary (MessagePack encoded)
     */
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let encode_res = to_vec(&self);
        if encode_res.is_err() {
            return Err("Unknown error encoding MessagePack package!".to_string());
        }
        Ok(
            encode_res.unwrap()
        )
    }
}

impl PackageAck {
    /**
     * Creates a new Package acknowledgement meta struct
     */
    pub fn new(package: &Package) -> Self {
        let instant = Instant::now();
        Self {
            cached_package: package.clone(),
            timestamp: instant,
            attempts: 1,
        }
    }
}
