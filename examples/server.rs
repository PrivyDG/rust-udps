extern crate udps;

use std::vec::Vec;
use std::string::String;

use udps::prelude::*;

fn main() {
    let mut clients: Vec<String> = Vec::new();
    
    let addr: String = "0.0.0.0:666".to_string();

    let endpoint_res = Endpoint::new(addr.clone(), 1024);
    if endpoint_res.is_err() {
        println!("Error creating endpoint!");
        std::process::exit(-127);
    }
    println!("Creating UDPS endpoint on {}...", addr);
    let mut endpoint = endpoint_res.unwrap();

    loop {
        let raw_res = endpoint.receive_from_raw();

        if raw_res.is_err() {
            println!("Received data, but some error occured!");
            continue;
        }

        let (raw_data, address) = raw_res.unwrap();
        let message_string = String::from_utf8(raw_data).unwrap_or("NULL".to_string());
                    
        if !clients.contains(&addr) {
            send_server_message(&mut endpoint , &mut clients, format!("New client from address {} connected!", &address));
            clients.push(address.clone());
        }

        match message_string.as_ref() {
            "EXIT" => {
                let index = clients.iter().position(|x| *x == address).unwrap();
                send_server_message(&mut endpoint, &mut clients, format!("User from address {} disconnected!", &address));
                continue;
            },
            "SHUTDOWN" => {
                send_server_message(&mut endpoint, &mut clients, "SHUTDOWN".to_string());
                break;
            },
            _ => {
                send_message(&mut endpoint, &mut clients, address.clone(), message_string);
            }
        };


    }

    println!("Server is shutting down...");
}

fn send_message(endpoint: &mut Endpoint, clients: &mut Vec<String>, from_addr: String, message: String) {
    let real_message = format!("{}> {}\r\n", &from_addr, &message);
    let data_slice: &[u8] = real_message.as_bytes();
    let data: Vec<u8> = Vec::from(data_slice);

    print!("{}", &real_message);

    for client in clients.iter() {
        if client != &from_addr {
            let send_res = endpoint.send_to_raw(client.clone(), &data);
            if send_res.is_err() {
                println!("Error sending message to endpoint {}!", client);
                continue;
            }
        }
    }
}

fn send_server_message(endpoint: &mut Endpoint, clients: &mut Vec<String>, message: String) {
    let real_message = format!("SERVER> {}\r\n", &message);
    let data_slice: &[u8] = real_message.as_bytes();
    let data: Vec<u8> = Vec::from(data_slice);

    print!("{}", &real_message);

    for client in clients.iter() {
        let send_res = endpoint.send_to_raw(client.clone(), &data);
        if send_res.is_err() {
            println!("Error sending message to endpoint {}!", client);
            continue;
        }
    }
}
