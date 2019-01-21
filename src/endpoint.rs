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
use std::time::{
    Instant,
    Duration
};

use rand::prelude::*;

use crate::prelude::*;

pub type Error = String;
pub type EndpointArc = Arc<Endpoint>;

pub struct EndpointConfig {
    pub address: String,
    pub buffer_size: u32,
    pub ack_interval: u64,
    pub ack_loop_time: u64,
    pub max_ack_attempts: u8,
    pub max_package_backlog: u32
}

pub struct Endpoint {
    pub config: EndpointConfig,
    pub socket: UdpSocket,
    pub running: AtomicBool,
    pub connection_list: RwLock<HashMap<u32, ConnectionArc>>,
    pub receive_thread: RwLock<Option<JoinHandle<()>>>,
    pub ack_thread: RwLock<Option<JoinHandle<()>>>,
    pub ack_list: RwLock<HashMap<u32, PackageAck>>
}

impl EndpointConfig {
    /**
     * Creates a new configuration with default values.
     */
    pub fn new(address: &String) -> Self {
        Self {
            address: address.clone(),
            buffer_size: 8192,
            ack_interval: 100,
            ack_loop_time: 250,
            max_ack_attempts: 10,
            max_package_backlog: 32
        }
    }
}

impl Endpoint {
    /**
     * Creates a new Endpoint and binds it to the current address.
     */
    pub fn new(config: EndpointConfig) -> Result<EndpointArc, Error> {
        let socket_res = UdpSocket::bind(&config.address);
        if socket_res.is_err() {
            return Err("Could not bind socket to address!".to_string());
        }
        let socket = socket_res.unwrap();

        let endpoint = Endpoint {
            running: AtomicBool::new(true),
            config: config,
            socket: socket,
            connection_list: RwLock::new(
                HashMap::new()
            ),
            receive_thread: RwLock::new(
                None
            ),
            ack_thread: RwLock::new(
                None
            ),
            ack_list: RwLock::new(
                HashMap::new()
            )
        };
        let endpoint_arc = Arc::new(
            endpoint
        );
        let endpoint_arc_receive = endpoint_arc.clone();
        let receive_handle = thread::spawn(move || {
            endpoint_arc_receive.receive_loop();
        });

        let endpoint_arc_ack = endpoint_arc.clone();
        let ack_handle = thread::spawn(move || {
            endpoint_arc_ack.ack_loop();
        });

        {
            let mut receive_thread = endpoint_arc.receive_thread.write().unwrap();
            *receive_thread = Some(receive_handle);
        }

        {
            let mut ack_thread = endpoint_arc.ack_thread.write().unwrap();
            *ack_thread = Some(ack_handle);
        }

        Ok(
            endpoint_arc
        )
    }

    /**
     * Stops the receive loop
     */
    pub fn stop(&mut self) {
        // Set boolean running to false so background loops stop after the current step
        self.running.store(false, Ordering::Relaxed);
        // Vector for caching ids to remvoe
        let mut removal_list = Vec::new();
        for (connection_id, _) in self.connection_list.read().unwrap().iter() {
            removal_list.push(*connection_id);
        }
        for id in removal_list.iter() {
            self.disconnect(id);
        }
        if self.receive_thread.read().unwrap().is_some() {
            let handle = self.receive_thread.write().unwrap().take().unwrap();
            handle.join().unwrap_or(());
        }
        if self.ack_thread.read().unwrap().is_some() {
            let handle = self.ack_thread.write().unwrap().take().unwrap();
            handle.join().unwrap_or(());
        }
    }

    /**
     * Connect to a another UDPS endpoint
*/
    pub fn connect(&mut self, addr: &String) -> Result<ConnectionArc, Error> {
        let mut package = Package::new_default();
        package.header.method_type = MethodType::Connect;
        package.header.ack = true;
        let conn_arc = Arc::new(
            Connection::new(addr, &package.header.connection_id)
        );
        {
            let mut connection_list = self.connection_list.write().unwrap();
            connection_list.insert(package.header.connection_id, conn_arc.clone());
        }
        Ok(
            conn_arc
        )
    }

    /**
     * Disconnect from another UDPS endpoint
*/
    pub fn disconnect(&mut self, connection_id: &u32) {
        let connection_opt = self.connection_list.write().unwrap().remove(connection_id);
        if connection_opt.is_some() {
            let mut package = Package::new_default();
            package.header.method_type = MethodType::Disconnect;
            package.header.connection_id = *connection_id;
            self.send(package).unwrap_or(0);
            let connection = connection_opt.unwrap();
            *connection.state.write().unwrap() = ConnectionState::Disconnected;
        }
    }

    /**
     * Receive loop
*/
    pub fn receive_loop(self: EndpointArc) {
        let mut running = true;
        while running {
            running = self.running.load(Ordering::Relaxed);
            let recv_res = self.receive();
            if recv_res.is_err() {
                continue;
            }
            let (package, _) = recv_res.unwrap();
            let exists = {
                let connections = self.connection_list.read().unwrap();
                connections.get(&package.header.connection_id).is_some()
            };
            if exists {
                continue;
            }
            let connection = {
                let connections = self.connection_list.read().unwrap();
                connections.get(&package.header.connection_id).unwrap().clone()
            };
            
            if package.header.ack {
                let mut response_package = Package::new_default();
                let data = conv_u32_to_bytes(&package.header.package_id);
                response_package.header.connection_id = connection.id;
                response_package.header.method_type = MethodType::Ack;
                response_package.data.clone_from_slice(&data);
                self.send(response_package).unwrap_or(0);
            }

        }
    }

    /**
     * Acknowledgement loop
     */
    pub fn ack_loop(self: EndpointArc) {
        let mut running = true;
        let mut timestamp_last = Instant::now();
        let mut removal_list = Vec::new();
        while running {
            for (package_id, package_ack) in self.ack_list.write().unwrap().iter_mut() {
                let current_time = Instant::now();
                if package_ack.attempts >= self.config.max_ack_attempts {
                    removal_list.push(package_ack.cached_package.header.package_id);
                    continue;
                }
                let elapsed_ms = current_time.duration_since(package_ack.timestamp).as_millis() as u64;
                if elapsed_ms >= self.config.ack_interval {
                    let package = package_ack.cached_package.clone();
                    let send_res = self.send(package);
                    if send_res.is_ok() {
                        package_ack.attempts += 1;
                        package_ack.timestamp = current_time.clone();
                    }
                }
            }
            for package_id in removal_list.iter() {
                self.ack_list.write().unwrap().remove(&package_id);
            }
            removal_list.clear();
            let now = Instant::now();
            let elapsed_ms = now.duration_since(timestamp_last).as_millis() as u64;
            if elapsed_ms <= self.config.ack_loop_time {
                std::thread::sleep(
                    Duration::from_millis(
                        self.config.ack_loop_time - elapsed_ms
                    )
                );
            }
            timestamp_last = now;
            running = self.running.load(Ordering::Relaxed);
        }
    }


    /**
     * Sends a package to another Endpoint
     */
    pub fn send(&self, package: Package) -> Result<usize, Error> {
        let exists = {
            let connections = self.connection_list.read().unwrap();
            connections.get(&package.header.connection_id).is_some()
        };
        if !exists {
            return Err("Connection unknown! Has it been dropped?".to_string());
        }
        
        let connection = {
            let connections = self.connection_list.read().unwrap();
            connections.get(&package.header.connection_id).unwrap().clone()
        };

        let create_ack;
        {
            create_ack = package.header.ack && !self.ack_list.read().unwrap().contains_key(&package.header.package_id);
        }

        if create_ack {
            let package_ack = PackageAck::new(&package);
            self.ack_list.write().unwrap().insert(package.header.package_id, package_ack);
        }

        let data: Vec<u8> = package.try_into()?;
        let send_res = self.socket.send_to(data.as_slice(), &connection.address);
        if send_res.is_err() {
            return Err(format!("Unknown error sending package to {} !", &connection.address));
        }
        Ok(
            send_res.unwrap()
        )
    }

    /**
     * Receives a package from another Endpoint
     */
    fn receive(&self) -> Result<(Package, String), Error> {
        let mut data: Vec<u8> = Vec::new();
        data.resize(self.config.buffer_size as usize, 0);
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
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        self.stop();
    }
}
