use std::convert::*;
use std::option::*;
use std::vec::*;

use rand::prelude::*;
use serde::*;
use rmps::*;

use crate::*;

#[derive(Serialize, Deserialize)]
pub struct Header {
    pub version: String,
    pub enc_type: EncType,
    pub crypt_type: CryptType,
    pub method_type: MethodType,
    pub package_id: u32,
    pub ack: bool,
    pub sequence_len: Option<u32>,
    pub sequence_ind: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub header: Header,
    pub data: Vec<u8>
}


#[derive(Serialize, Deserialize)]
pub enum EncType {
    Raw = 0,
    ZIP,
    LZO    
}

#[derive(Serialize, Deserialize)]
pub enum CryptType {
    None = 0,
    Asymm,
    Symm
}

#[derive(Serialize, Deserialize)]
pub enum MethodType {
    Connect = 0,
    Disconnect,
    Ack,
    AsymmKey,
    SymmKey,
    Data,
}

impl TryFrom<Vec<u8>> for Package {
    type Error = String;

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