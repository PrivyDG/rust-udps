use std::net::*;
use std::ops::{
    Drop,
    DerefMut,
    Deref
};
use std::convert::*;
use std::thread;
use std::thread::JoinHandle;
use std::collections::*;
use std::sync::*;
use std::sync::atomic::*;
use std::cell::RefCell;
use std::time::Duration;

use rand::prelude::*;

use crate::prelude::*;

pub struct Endpoint {
    pub address: String,
    buffer_size: i32,
    read_timeout: i32,
    socket: UdpSocket,
    pub running: AtomicBool,
    pub connections: HashMap<u32, Connection>,
    pub receive_thread: Option<JoinHandle<()>>
}

pub type Error = String;
pub type EndpointArc = Arc<RwLock<Endpoint>>;

impl Endpoint {
    /**
     * Creates a new Endpoint and binds it to the current address.
    */
    pub fn new(addr: String, buffer_size: i32, read_timeout: i32) -> Result<EndpointArc, Error> {
        let socket_res = UdpSocket::bind(&addr);
        if socket_res.is_err() {
            return Err("Could not bind socket to address!".to_string());
        }
        let socket = socket_res.unwrap();
        socket.set_read_timeout(
            Some(
                Duration::from_millis(read_timeout as u64)
            )
        ).unwrap();
        let endpoint = Endpoint {
            address: addr,
            buffer_size: buffer_size,
            read_timeout: read_timeout,
            socket: socket,
            running: AtomicBool::new(true),
            connections: HashMap::new(),
            receive_thread: None,
        };
        let mut endpoint_arc = Arc::new(
            RwLock::new(
                endpoint
            )
        );
        let endpoint_arc_copy = endpoint_arc.clone();
        let join_handle = thread::spawn(move || {
            Endpoint::receive_loop(endpoint_arc_copy);
        });
        {
            endpoint_arc.write().unwrap().receive_thread = Some(join_handle);
        }
        Ok(
            endpoint_arc
        )
    }

    /**
     * Stops the receive loop
     */
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if self.receive_thread.is_some() {
            let handle = self.receive_thread.take().unwrap();
            handle.join().unwrap_or(());
        }
    }

    /**
     * Connect to a serving UDPS endpoint
    */
    pub fn connect(&mut self, addr: &String, connection_timeout: i32) -> Result<&Connection, Error> {
        let mut package = Package::default();
        package.header.method_type = MethodType::Connect;
        package.header.ack = true;
        let conn = Connection::new(addr, &package.header.connection_id);
        self.connections.insert(package.header.connection_id, conn);
        let conn_ref: &Connection = self.connections.get(&package.header.connection_id).unwrap();
        let _ = self.send_to(&conn_ref.addr, package)?;
        thread::sleep(
            Duration::from_millis(connection_timeout as u64)
        );
        Ok(
            conn_ref
        )
    }

    /**
     * Disconnect from another UDPS endpoint
    */
    pub fn disconnect(&mut self, _connection: &Connection) -> Connection {
        let mut package = Package::default();
        package.header.method_type = MethodType::Disconnect;
        package.header.connection_id = _connection.id;
        let _ = self.send_to(&_connection.addr, package).unwrap_or(0);
        self.connections.remove(&_connection.id).unwrap()
    }

    /**
     * Receive loop
     */
    pub fn receive_loop(endp_arc: EndpointArc) {
        let mut running = true;
        while running {
            let package;
            {
                let endp = endp_arc.read().unwrap();
                let recv_res = endp.receive_from();
                running = endp.running.load(Ordering::Relaxed);
                if recv_res.is_err() {
                    continue;
                }
                package = recv_res.unwrap().0;
            }
            {
                let mut endp = endp_arc.write().unwrap();
                let conn_opt = endp.connections.get_mut(&package.header.connection_id);
                if conn_opt.is_none() {
                    continue;
                }
                let conn = conn_opt.unwrap();
                conn.push_package(package);
            }
        }
    }

    /**
     * Sends a package to another Endpoint
    */
    pub fn send_to(&self, addr: &String, package: Package) -> Result<usize, Error> {
        let real_data: Vec<u8> = package.try_into()?;
        let size_res = self.socket.send_to(real_data.as_slice(), &addr);
        if size_res.is_err() {
            return Err("Error sending package!".to_string());
        }
        Ok(
            size_res.unwrap()
        )
    }

    /**
     * Receives a package from another Endpoint
    */
    pub fn receive_from(&self) -> Result<(Package, String), Error> {
        let mut data: Vec<u8> = Vec::new();
        data.resize(self.buffer_size as usize, 0);
        let recv_res = self.socket.recv_from(data.as_mut_slice());
        if recv_res.is_err() {
            return Err("Error receiving!".to_string());
        }
        let (real_size, addr) = recv_res.unwrap();
        data.resize(real_size, 0);
        let package = Package::try_from(data)?;
        Ok(
            (package, addr.to_string())
        )   
    }

    /**
     * Sends raw data to another Endpoint
    */
    pub fn send_to_raw(&self, addr: String, data: &Vec<u8>) -> Result<usize, Error> {
        let mut package = Package::default();
        package.data = data.to_owned();
        let real_data: Vec<u8> = package.try_into()?;
        let size_res = self.socket.send_to(real_data.as_slice(), &addr);
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
        data.resize(self.buffer_size as usize, 0);
        let recv_res = self.socket.recv_from(data.as_mut_slice());
        if recv_res.is_err() {
            return Err(format!("Error: Unknown"));
        }
        let (real_size, addr) = recv_res.unwrap();
        data.resize(real_size, 0);
        println!("Received data of size {} !", &real_size);
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
