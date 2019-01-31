use std::net::*;
use std::io::{
    stdout,
    Write
};
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
use openssl::rsa::*;
use openssl::pkey::*;
use crate::prelude::*;

/**
 * Convencience typedef, we use simple messages as errors
 */
pub type Error = String;
/**
 * Convenience typedef, Arc of Endpoint
 */
pub type EndpointArc = Arc<Endpoint>;

/**
 * Endpoint configuration
 */
pub struct EndpointConfig {
    /**
     * The address the endpoint should bind to
     */
    pub address: String,
    /**
     * The buffer size used when retrieving data
     */
    pub buffer_size: u32,
    /**
     * The amount of time (in ms) the socket will
     * stop blocking after not receiving data
     */
    pub read_timeout: u64,
    /**
     * Unused
     */
    pub ack_interval: u64,
    /**
     * Interval of the Acknowledgement loop running in the background
     */
    pub ack_loop_time: u64,
    /**
     * Maximum number of Acknowledgement attempts
     */
    pub max_ack_attempts: u8,
    /**
     * Maximum length of package queue
     */
    pub max_package_backlog: u32,
    /**
     * Private RSA key, used for transmitting the symmetric key
     */
    pub private_key: Rsa<Private>,
    /**
     * Private AES key, used for encrypting outgoing messages
     */
    pub secret_key: Vec<u8>
}

/**
 * Endpoint struct.
 * Represents an endpoint.  
 * An endpoint binds to a single address and port - you can let the  
 * OS choose one for you by picking ":0". An endpoint can have multiple connections, retrieved  
 * via the `collect_new_connections()` method, and initiated via the `connect` method.  
 * Packages are retrieved asynchonously in a background thread and placed in a queue owned by  
 * the corresponding connection.  
 * It automatically resends packages every `ack_loop_time` ms that have the `ack` flag set,  
 * a maximum `max_ack_attempts` of times.
 * 
 */
pub struct Endpoint {
    /**
     * Configuration
     */
    pub config: EndpointConfig,
    /**
     * Internal: UDP socket
     */
    pub socket: UdpSocket,
    /**
     * True if socket is currently running
     */
    pub running: AtomicBool,
    /**
     * List of connections
     */
    pub connection_list: RwLock<HashMap<u32, ConnectionArc>>,
    /**
     * List of packages to resend in the background
     */
    pub ack_list: RwLock<HashMap<u32, PackageAck>>,
    /**
     * List of new connections
     */
    pub new_connection_list: RwLock<Vec<ConnectionArc>>,
    /**
     * Thread handle for the thread receiving data in the background
     */
    pub receive_thread: RwLock<Option<JoinHandle<()>>>,
    /**
     * Thread handle for the thread resending packages until
     * they are received
     */
    pub ack_thread: RwLock<Option<JoinHandle<()>>>
}

impl EndpointConfig {
    /**
     * ## Creates a new configuration with default values.
     */
    pub fn new(address: &String) -> Self {
        let rsa = Rsa::generate(2048).unwrap();
        Self {
            address: address.clone(),
            buffer_size: 8192,
            read_timeout: 1000,
            ack_interval: 200,
            ack_loop_time: 1000,
            max_ack_attempts: 20,
            max_package_backlog: 32,
            private_key: rsa,
            secret_key: generate_random_bytes(32)
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

        socket.set_read_timeout(
            Some(
                Duration::from_millis(
                    config.read_timeout
                )
            )
        ).unwrap_or(());

        let endpoint = Endpoint {
            running: AtomicBool::new(true),
            config: config,
            socket: socket,
            new_connection_list: RwLock::new(
                Vec::new()
            ),
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
     * Stops the socket and all background threads.
     */
    pub fn stop(&self) {
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
     * Connect to a another UDPS endpoint.  
     * This method will send a connection request to the supplied  
     * address and return a `ConnectionArc` corresponding to this connection.
     */
    pub fn connect(&self, addr: &String) -> Result<ConnectionArc, Error> {
        let stdout = stdout();
        //writeln!(&mut stdout.lock(), "Connecting UDPS endpoint to {}", addr);
        let mut package = Package::new_default();
        package.header.method_type = MethodType::Connect;
        package.header.ack = true;
        let connection_id = package.header.connection_id;

        let conn_arc = Arc::new(Connection::new(addr, &connection_id));
        {
            let mut connection_list = self.connection_list.write().unwrap();
            connection_list.insert(connection_id, conn_arc.clone());
        }

        *conn_arc.state.write().unwrap() = ConnectionState::Connected;

        self.send(package).unwrap();

        Ok(
            conn_arc
        )
    }

    /**
     * Disconnect from another UDPS endpoint
     */
    pub fn disconnect(&self, connection_id: &u32) {
        let exists = {
            let connections = self.connection_list.read().unwrap();
            connections.get(connection_id).is_some()
        };
        if exists {
            let mut package = Package::new_default();
            package.header.method_type = MethodType::Disconnect;
            package.header.connection_id = *connection_id;
            self.send(package).unwrap_or(0);
            let connection = self.connection_list.write().unwrap().remove(connection_id).unwrap();
            *connection.state.write().unwrap() = ConnectionState::Disconnected;
        }
    }

    /**
     * Receive loop.
     * This is started automatically in the background,
     * and should not be called manually.
     */
    pub fn receive_loop(&self) {
        let stdout = stdout();
        let mut running = true;
        while running {
            running = self.running.load(Ordering::Relaxed);
            let recv_res = self.receive();
            if recv_res.is_err() {
                continue;
            }
            let (package, addr) = recv_res.unwrap();
            self.handle_package(addr, package);
        }
        //writeln!(&mut stdout.lock(), "Shutting down receive_loop");
    }

    /**
     * Internal method for handling a specific package after receival.
     */
    fn handle_package(&self, addr: String, package: Package) {
        let stdout = stdout();
        //writeln!(&mut stdout.lock(), "Package from {} !", addr);
        // Check if there exists a connection:
        let exists = {
            let connections = self.connection_list.read().unwrap();
            connections.get(&package.header.connection_id).is_some()
        };
        let conn_arc: ConnectionArc;
        // Connection does not exist. It is either a new connection request or an
        // unknown/unauthorized connection
        if !exists {
            // If this package is not a connection request and from and unknown connection - drop it!
            if package.header.method_type != MethodType::Connect {
                return;
            }
            // The package is a connection request. Create a new connection.
            let new_conn_arc = Arc::new(
                Connection::new(&addr, &package.header.connection_id)
            );

            *new_conn_arc.state.write().unwrap() = ConnectionState::Connected;

            let mut new_connections = self.new_connection_list.write().unwrap();
            new_connections.push(new_conn_arc.clone());

            // Save this connection in the internal connection list.
            let mut connections = self.connection_list.write().unwrap();
            connections.insert(package.header.connection_id, new_conn_arc);
            // Return a clone of the Arc to this list item
            conn_arc = connections.get(&package.header.connection_id).unwrap().clone();
        }
        // Connection exists
        else {
            let connections = self.connection_list.read().unwrap();
            conn_arc = connections.get(&package.header.connection_id).unwrap().clone();
        }
        
        // NEXT: Handle package acknowledgement
        // If the incoming package has the `ack` flag set,
        // Immediately send a response acknowledging the package.
        if package.header.ack {
            //writeln!(&mut stdout.lock(), "Package wants ack. Sending it.");
            let mut response_package = Package::new_default();
            let data = conv_u32_to_bytes(&package.header.package_id);
            response_package.header.connection_id = conn_arc.id;
            response_package.header.method_type = MethodType::Ack;
            response_package.data.resize(data.len(), 0);
            response_package.data.clone_from_slice(&data);
            //writeln!(&mut stdout.lock(), "Sending package ack.");
            self.send(response_package).unwrap();
        }

        match package.header.method_type {
            MethodType::Ack => {
                self.handle_ack(conn_arc, package);
                return;
            },
            MethodType::Connect => {
                return;
            },
            MethodType::Disconnect => {
                *conn_arc.state.write().unwrap() = ConnectionState::Disconnected;
                let mut connection_list = self.connection_list.write().unwrap();
                connection_list.remove(&conn_arc.id);
                return;
            }
            _ => {
                // For now, just pass the package to the connection.
                // It will automatically be dropped if its a duplicate.
                conn_arc.push_package(package);
            }
        };
    }

    /**
     * Acknowledgement loop.
     * This is started automatically in the background,  
     * and should not be called manually.
     */
    pub fn ack_loop(&self) {
        let stdout = stdout();
        let mut iteration_ms = 0u64;
        let mut remove_list = Vec::new();
        let mut attempt_increase_list = Vec::new();
        //writeln!(&mut stdout.lock(), "starting ack_loop");
        loop_at!((1000 / self.config.ack_loop_time), iteration_ms, {
            //writeln!(&mut stdout.lock(), "ack_loop iteration");
            if !self.running.load(Ordering::Relaxed) {
                break;
            } 
            // Read actions
            {
                let ack_list = self.ack_list.read().unwrap();
                for (package_id, package_ack) in ack_list.iter() {
                    if package_ack.attempts >= self.config.max_ack_attempts {
                        remove_list.push(*package_id);
                        continue;
                    }
                    let package = package_ack.cached_package.clone();
                    let send_res = self.send(package);
                    if send_res.is_err() {
                        //writeln!(&mut stdout.lock(), "Error sending ack package!");
                        continue;
                    }
                    attempt_increase_list.push(*package_id);
                }
            }
            // Write actions
            {
                let mut ack_list = self.ack_list.write().unwrap();
                for package_id in attempt_increase_list.iter() {
                    let mut package_ack = ack_list.get_mut(package_id).unwrap();
                    package_ack.attempts += 1;
                }
                for package_id in remove_list.iter() {
                    let package_ack = ack_list.remove(package_id).unwrap();
                    if package_ack.cached_package.header.method_type == MethodType::Connect {
                        // Update connection state to Disconnected
                        {
                            let mut connections = self.connection_list.write().unwrap();
                            let connection_res = connections.remove(&package_ack.cached_package.header.connection_id);
                            if connection_res.is_some() {
                                let connection = connection_res.unwrap();
                                *connection.state.write().unwrap() = ConnectionState::Disconnected;
                            }
                        }
                    }
                }
            }
            // Clear vectors
            remove_list.clear();
            attempt_increase_list.clear();
        });

        //writeln!(&mut stdout.lock(),  "Shutting down ack_loop");
    }

    /**
     * Internal function for handling incoming packages   
     * with the `Ack` method.
     */
    fn handle_ack(&self, conn: ConnectionArc, package: Package) {
        let package_ack_res = {
            let mut acks = self.ack_list.write().unwrap();
            let id = conv_slice_to_u32(package.data.as_slice());
            acks.remove(&id)
        };
        if package_ack_res.is_none() {
            return;
        }
        let package_ack = package_ack_res.unwrap();
        match package_ack.cached_package.header.method_type {
            MethodType::Connect => {
                *conn.state.write().unwrap() = ConnectionState::Connected;
            },
            _ => {}
        };
    }


    /**
     * Sends a package, and returns the sent size.
     */
    pub fn send(&self, package: Package) -> Result<usize, Error> {
        let stdout = stdout();
        //writeln!(&mut stdout.lock(), "Sending package!");
        let exists = {
            let connections = self.connection_list.read().unwrap();
            connections.get(&package.header.connection_id).is_some()
        };
        if !exists {
            return Err("Connection unknown! Has it been dropped?".to_string());
        }
        //writeln!(&mut stdout.lock(), "Connection exists!");

        let connection = {
            let connections = self.connection_list.read().unwrap();
            connections.get(&package.header.connection_id).unwrap().clone()
        };

        let create_ack = { 
            package.header.ack && 
            !self.ack_list.read().unwrap().contains_key(&package.header.package_id)
        };

        if create_ack {
            //writeln!(&mut stdout.lock(), "Adding PackageAck!");
            let package_ack = PackageAck::new(&package);
            self.ack_list.write().unwrap().insert(package.header.package_id, package_ack);
        }
        //writeln!(&mut stdout.lock(), "Sending package!");

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
     * Receives a package, and returns it and the sender address.
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

    pub fn collect_connections(&self) -> Vec<ConnectionArc> {
        let connections = self.connection_list.read().unwrap();
        connections.iter().map(|(_, c)| c.clone()).collect()
    }

    pub fn collect_new_connections(&self) -> Vec<ConnectionArc> {
        let mut new_connections = self.new_connection_list.write().unwrap();
        let ret = new_connections.clone();
        new_connections.clear();
        ret
    }
}

/**
 * Implementing `Drop`, which simply  
 * stops the endpoint.
 */
impl Drop for Endpoint {
    fn drop(&mut self) {
        self.stop();
    }
}
