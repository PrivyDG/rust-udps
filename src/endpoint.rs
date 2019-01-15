use std::net::*;
use std::ops::Drop;
use std::convert::*;

use rand::prelude::*;

use crate::prelude::*;

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
    pub fn send_to_raw(&self, addr: String, data: &Vec<u8>) -> Result<usize, Error> {
        let package = Package {
            header: Header {
                version: format!("{}.{}.{}", &VERSION_MAJOR, &VERSION_MINOR, &VERSION_PATCH),
                enc_type: EncType::Raw,
                crypt_type: CryptType::None,
                method_type: MethodType::Data,
                package_id: thread_rng().next_u32(),
                ack: true,
                sequence_len: None,
                sequence_ind: None
            },
            data: data.to_owned()
        };
        let real_data: Vec<u8> = package.try_into()?;
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
        let mut data: Vec<u8> = Vec::new();
        data.reserve(self.buffer_size as usize);
        let recv_res = self.socket.recv_from(data.as_mut_slice());
        if recv_res.is_err() {
            return Err(format!("Error: Unknown"));
        }
        let (real_size, addr) = recv_res.unwrap();
        data.resize(real_size, 0);
        let package = Package::try_from(data)?;
        Ok(
            (package.data, addr.to_string())
        )
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {

    }
}