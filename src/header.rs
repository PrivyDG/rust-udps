use std::convert::*;
use std::option::*;
use std::vec::*;

use byteorder::{LittleEndian, ReadBytesExt};
use rand::prelude::*;

use crate::*;

/**
 * Header struct
*/
pub struct Header {
    pub version: String, // 4B
    pub version_raw: [u8; 3], // 3B
    pub method: Method, // 1B
    pub conn_id: u32, // 4B
    pub pack_id: u32, // 4B
    pub ack: bool, // 1B
    pub data_len: u32,
    pub sequence_id: Option<u32>,
    pub sequence_index: Option<u32>,
}

/**
 * Package method
*/
#[derive]
pub enum Method {
    Ping = 0,
    PingResponse,
    Connect,
    Disconnect,
    Ack,
    PublicKey,
    SecretKey,
    Data,
    DataSeq,
}

impl From<&u8> for Method {
    fn from(byte: &u8) -> Self {
        match byte {
            0 => {
                return Method::Ping;
            },
            1 => {
                return Method::PingResponse;
            },
            2 => {
                return Method::Connect;
            },
            3 => {
                return Method::Disconnect;
            },
            4 => {
                return Method::Ack;
            },
            5 => {
                return Method::PublicKey;
            },
            6 => {
                return Method::SecretKey;
            },
            7 => {
                return Method::Data;
            },
            8 => {
                return Method::DataSeq;
            },
            _ => {
                return Method::Ping;
            }
        }
    }
}

impl Into<u8> for Method {
    fn into(self) -> u8 {
        match self {
            Method::Ping => {
                return 0;
            },
            Method::PingResponse => {
                return 1;
            },
            Method::Connect => {
                return 2;
            },
            Method::Disconnect => {
                return 3;
            },
            Method::Ack => {
                return 4;
            },
            Method::PublicKey => {
                return 5;
            },
            Method::SecretKey => {
                return 6;
            },
            Method::Data => {
                return 7;
            },
            Method::DataSeq => {
                return 8;
            },
            _ => {
                return 0;
            }
        }
    }
}

impl Header {
    pub fn new(conn_id: &u32) -> Self {
        let mut rng = thread_rng();
        Header {
            version: format!("{}.{}.{}", &VERSION_MAJOR, &VERSION_MINOR, &VERSION_PATCH),
            version_raw: [VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH],
            method: Method::Ping,
            conn_id: conn_id.clone(),
            pack_id: rng.gen(),
            ack: true,
            data_len: 0,
            sequence_id: None,
            sequence_index: None,
        }
    }

    pub fn byte_len(&self) -> u8 {
        match self.method {
            Method::DataSeq => {
                return 29;
            },
            _ => {
                return 21;
            }
        }
    }
}

pub fn get_u32(slice: &[u8]) -> Option<u32> {
    if slice.len() < 4 {
        return None;
    }
    let arr = [
        slice[0],
        slice[1],
        slice[2],
        slice[3]
    ];
    return Some(
        u32::from_le_bytes(arr)
    );
}

impl TryInto<Vec<u8>> for Header {
    type Error = String;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let ret: Vec<u8> = Vec::new();
        
        Ok(
            Vec::new()
        )
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Header, Self::Error> {
        let mut  version_raw = [0u8; 3];
        version_raw.clone_from_slice(&data[4..6]);
        let method = Method::from(&data[7]);
        let conn_id = get_u32(&data[8..11]).unwrap();
        let pack_id = get_u32(&data[12..15]).unwrap();
        let ack = data[16] != 0;
        let data_len = get_u32(&data[17..20]).unwrap();

        let mut sequence_id: Option<u32> = None;
        let mut sequence_index: Option<u32> = None;

        match method {
            Method::DataSeq => {
                sequence_id = get_u32(&data[21..24]);
                sequence_index = get_u32(&data[25..28]);
            },
            _ => {}
        }

        Ok(
            Header {
                version: format!("{}.{}.{}", version_raw[0], version_raw[1], version_raw[2]),
                version_raw: version_raw,
                method: method,
                conn_id: conn_id,
                pack_id: pack_id,
                ack: ack,
                data_len: data_len,
                sequence_id: sequence_id,
                sequence_index: sequence_index
            }
        )
    }
}
