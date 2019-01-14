use std::convert::*;
use std::option::*;
use std::vec::*;

use byteorder::{LittleEndian, ReadBytesExt};
use rand::prelude::*;
use serde::*;
use bson::*;
use bson::spec::*;

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
        let bson = bson::Bson::from(
            (BinarySubtype::Generic, data)
        );
        let decode_res = from_bson::<Self>(bson);
        if decode_res.is_err() {
            return Err("Unknown BSON decode error!".to_string());
        }
        Ok (
            decode_res.unwrap()
        )
    }
}

impl TryInto<Vec<u8>> for Package {
    type Error = String;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let encode_res = to_bson::<Self>(&self);
        if encode_res.is_err() {
            return Err("Unknown BSON encode error!".to_string());
        }
        let bson = encode_res.unwrap();
        if let Bson::Binary(BinarySubtype::Generic, data) = bson {
            return Ok(data);
        }
        Err("Unimplemented.".to_string())
    }
}