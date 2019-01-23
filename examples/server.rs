use std::vec::Vec;
use std::string::String;
use std::sync::*;
use std::collections::*;
use std::sync::atomic::*;
use std::io::*;
use std::iter::IntoIterator;
use std::ops::{
    Deref,
    DerefMut
};

use udps::prelude::*;

static RUNNING: AtomicBool = AtomicBool::new(false);

fn main() {
    let mut address = String::new();
    println!("Enter address the server should bind to.");
    print!(">> ");
    stdout().flush().unwrap();
    stdin().read_line(&mut address).unwrap();
    address.pop();
    let config = EndpointConfig::new(&address);
    let endpoint_res = Endpoint::new(config);
    if endpoint_res.is_err() {
        println!("Error creating endpoint on {} !", &address);
        std::process::exit(-1);
    }
    let endpoint = endpoint_res.unwrap();
    let mut connections: Box<Vec<ConnectionArc>> = Box::new(Vec::new());
    let mut messages: Vec<String> = Vec::new();
    RUNNING.store(true, Ordering::Relaxed);
    println!("Starting stuff");
    stdout().flush().unwrap();
    loop_at!(10, {
        if !RUNNING.load(Ordering::Relaxed) {
            break;
        }

        let mut new_connections = endpoint.collect_new_connections();
        for connection in new_connections {
            println!("New connection!");
            connections.push(connection);
        }

        for connection in connections.iter() {
            println!("Handling connection...");
            let packages = connection.collect_packages();

            for package in packages.iter() {
                let data = package.data.clone();
                let message = String::from_utf8_lossy(data.as_slice()).to_string();
                println!(">> {}", message);
                messages.push(message);
            }
        }

        for connection in connections.iter() {
            for message in messages.iter() {
                let data = message.as_bytes();
                let mut package = Package::new_default();
                package.header.method_type = MethodType::Data;
                package.header.connection_id = connection.id;
                package.data.resize(data.len(), 0);
                package.data.clone_from_slice(data);

                endpoint.send(package).unwrap();
            }
        }
        
        messages.clear();
    });
}
