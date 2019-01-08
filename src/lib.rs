use std::vec::Vec;

/**
 * Endpoint logic to send and receive data with the UDPS protocol
*/
pub mod endpoint;

/**
 * Connection logic to store connection state for endpoints
*/
pub mod connection;

/**
 * Prelude module to reexport everything
*/
pub mod prelude {
    pub use crate::VERSION_MAJOR;
    pub use crate::VERSION_MINOR;
    pub use crate::VERSION_PATCH;
    pub use crate::Header;
    pub use crate::endpoint::*;
}

pub static VERSION_MAJOR: u8 = 0;
pub static VERSION_MINOR: u8 = 1;
pub static VERSION_PATCH: u8 = 1;

pub struct Header {
    pub version: String,
    pub application_id: i32,
    pub enc_type: EncType,
    pub package_type: PackageType,
    pub package_id: i32,
    pub package_sequence: i32
}

impl Header {
    pub fn default_raw(application_id: i32) -> Header {
        Header {
            version: format!("{}.{}.{}", &VERSION_MAJOR, &VERSION_MINOR, &VERSION_PATCH),
            application_id: application_id,
            enc_type: EncType::Raw,
            package_type: PackageType::DataBin,
            package_id: 0,
            package_sequence: 0
        }
    }

    pub fn as_u8(&self) -> Vec<u8> {
        let header_name = b"UDPS";
        let header_version = [VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH];
        /**
        let header_app_id =  self.application_id.clone();
        let header_enc_type: u8 = self.enc_type as u8;
        let header_pack_type: u8 = self.package_type as u8;
        let header_pack_id = self.package_id.clone();
        let header_pack_seq = self.package_sequence.clone();
        */

        let mut header: Vec<u8> = Vec::new();
        header.extend_from_slice(header_name);
        header.extend_from_slice(&header_version);
        /**
        header.extend_from_slice();
        header.extend_from_slice(&header_enc_type);
        header.extend_from_slice(&header_pack_type);
        header.extend_from_slice(&header_pack_id);
        header.extend_from_slice(&header_pack_seq);
        */
        header
    }
}

#[derive(Copy, Clone)]
pub enum EncType {
    Raw = 0,
    Assymetric,
    Symmetric
}

#[derive(Copy, Clone)]
pub enum PackageType {
    Connect = 0,
    Ack,
    Request,
    DataBin,
}
