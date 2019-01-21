use std::io::*;
use std::sync::atomic::*;
use std::thread;
use std::sync::*;
use std::result::*;

use udps::prelude::*;

static mut running: AtomicBool = AtomicBool::new(false);

fn main() {
    let mut address = String::new();
    let mut cont = true;

    while cont {
        println!("Please enter the address and port to connect to.");
        print!(">> ");
        let _ = stdout().flush();
        let read_res = stdin().read_line(&mut address);
        cont = read_res.is_err();
    }
    address.pop();

    println!("Connecting to {} .", &address);
    let endpoint_res = Endpoint::new("127.0.0.1:8888".to_string(), 1024, 1000);
    if endpoint_res.is_err() {
        let error_message = match endpoint_res {
            Err(err_message) => err_message,
            Ok(_) => "Unknown Error!".to_string()
        };
        println!("{}", error_message);
        std::process::exit(-127);
    }

    let endpoint = endpoint_res.unwrap();
    println!("Created endpoint");
    
    unsafe {
        let t_endpoint = endpoint.clone();
        running.store(true, Ordering::SeqCst);
        
        let join_handle = thread::spawn(move || {
            println!("Receive thread: Starting receive_loop");
            receive_loop(endpoint);
        });

        let mut line = String::new();
        thread::sleep_ms(500);
        println!("Main thread: Starting send_loop");
        while running.load(Ordering::SeqCst) {
            println!("Reading new line...");
            print!(">> ");
            let _ = stdout().flush();
            let read_res = stdin().read_line(&mut line);
            if read_res.is_err() {
                continue;
            }
            if line == "EXIT" {
                println!("User pressed \"EXIT\".");
                break;
            } else {
                send_message(&endpoint, &address, &line);
            }
        }
        println!("send_loop ended!");
        running.store(false, Ordering::SeqCst);
        join_handle.join().unwrap();
    }

    println!("Server shutting down...");
}

fn receive_loop(endpoint: EndpointArc) {
    unsafe {
        while running.load(Ordering::SeqCst) {
            let recv_res = endpoint.receive_from_raw();
            if recv_res.is_err() {
                continue;
            }
            let (data, addr) = recv_res.unwrap();
            let message = String::from_utf8_lossy(data.as_slice());
            println!("{}", message);
        }
    }
}

fn send_message(endpoint: EndpointArc, address: &String, message: &String) {
    println!("Sending message to {} ...", address);
    let data_slice = message.as_bytes();
    let data = Vec::from(data_slice);
    println!("Data size: {} ...", data.len());
    let send_res = endpoint.send_to_raw(address.clone(), &data);
    if send_res.is_err() {
        println!("Error sending message.");
    }
}
