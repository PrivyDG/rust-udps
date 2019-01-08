use std::net::*;
use std::ops::Drop;

use crate::*;

pub struct Endpoint {
    pub address: String,
    buffer_size: i32,
    socket: UdpSocket
}

pub type Error = String;

impl Endpoint {
    /**
     * Creates a new Endpoint and binds it to the current address.
     */
    pub fn new(addr: String, buffer_size: i32) -> Result<Endpoint, Error> {
        let socket_res = UdpSocket::bind(addr.clone());
        if socket_res.is_err() {
            return Err(format!("Error: Could not bind to address {}! Port taken?", addr));
        }
        Ok(
            Endpoint {
                address: addr,
                buffer_size: buffer_size,
                socket: socket_res.unwrap()
            }
        )
    }

    /**
     * Connect to a serving UDPS endpoint
     */
    pub fn connect(&self, addr: String, buffer_size: i32) -> Result<(), Error> {
        Err(
            "Error: Not implemented!".to_string()
        )
    }

    

    /**
     * Sends raw data to another Endpoint
     */
    pub fn send_to_raw(&self, addr: String, data: &[u8]) -> Result<usize, Error> {
        let header_udps = b"UDPS";
        let header_version = [ VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH ];
        let mut real_data = Vec::new();
        real_data.extend_from_slice(header_udps);
        real_data.extend_from_slice(&header_version);
        real_data.extend_from_slice(data);
        let size_res = self.socket.send_to(&real_data, &addr);
        if size_res.is_err() {
            return Err(format!("Error: Could not send raw data to address {}!", &addr));
        }
        Ok(size_res.unwrap())
    }

    /**
     * Receives raw data
     */
    pub fn receive_from_raw(&self) -> Result<(Vec<u8>, String), Error> {
        let header_udps = b"UDPS";
        let header_version = [ VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH ];
        let mut data: Vec<u8> = Vec::new();
        data.reserve(self.buffer_size as usize);
        let recv_res = self.socket.recv_from(data.as_mut_slice());
        if recv_res.is_err() {
            return Err(format!("Error: Unknown"));
        }
        let (real_size, addr) = recv_res.unwrap();
        data.resize(real_size, 0);
        
        // Check for UDPS header
        if &data[..3] != header_udps {
            return Err("Error: Received raw UDP packet on UDPS endpoint!".to_string());
        }
        // Check for correct UDPS version
        if &data[4..6] != &header_version {
            return Err(format!("Error: UDPS version mismatch ({}.{}.{} != {}.{}.{})!", &VERSION_MAJOR, &VERSION_MINOR, &VERSION_PATCH, &data[4], &data[5], &data[6]))
        }

        data = data[6..].to_vec();
        Ok(
            (data, addr.to_string())
        )
    }
}
