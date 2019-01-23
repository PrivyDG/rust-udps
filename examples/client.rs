use std::io::*;
use std::sync::atomic::*;
use std::thread;
use std::sync::*;
use std::result::*;
use std::string::*;

use udps::prelude::*;

pub static RUNNING: AtomicBool = AtomicBool::new(false);

fn main() {
    println!("Enter address to connect to.");
    print!(">> ");
    let mut address = String::new();
    stdout().flush().unwrap();
    stdin().read_line(&mut address).unwrap();
    address.pop();
    let own_address = "127.0.0.1:0".to_string();
    let config = EndpointConfig::new(&own_address);
    let endpoint_res = Endpoint::new(config);
    if endpoint_res.is_err() {
        println!("ERROR! Could not create endpoint! Terminating program.");
        std::process::exit(-1);
    }
    println!("Created endpoint.");
    let endpoint = endpoint_res.unwrap();
    RUNNING.store(true, Ordering::Relaxed);
    println!("Setting runstate.");
    let endpoint_copy = endpoint.clone();
    let connection = endpoint.connect(&address).unwrap();
    let connection_copy = connection.clone();
    println!("Created Arc copies.");
    let join_handle = std::thread::spawn(move || {
        receive_loop(endpoint_copy, connection_copy);
    });
    println!("Starting main send thread.");
    while RUNNING.load(Ordering::Relaxed) {
        print!(">> ");
        stdout().flush().unwrap();
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        line.trim();
        line.pop();
        match line.as_str() {
            "EXIT" => {
                RUNNING.store(false, Ordering::Relaxed);
                break;
            },
            _ => {
                let mut package = Package::new_default();
                package.header.connection_id = connection.id;
                package.header.method_type = MethodType::Data;
                let data = line.as_bytes();
                package.data.resize(data.len(), 0);
                package.data.clone_from_slice(data);
                endpoint.send(package).unwrap();
            }
        };
    }
    let stdout = stdout();
    writeln!(&mut stdout.lock(), "Shutting down client.").unwrap();
    writeln!(
        &mut stdout.lock(),
        "Waiting for receive_loop thread to shut down..."
    )
    .unwrap();
    join_handle.join().unwrap();
    writeln!(&mut stdout.lock(), "OK").unwrap();
    writeln!(&mut stdout.lock(), "Waiting for endpoint to shut down...").unwrap();
    endpoint.stop();
    writeln!(&mut stdout.lock(), "OK").unwrap();
    writeln!(&mut stdout.lock(), "Shutdown finished and clean.").unwrap();
}

fn receive_loop(endpoint: EndpointArc, connection: ConnectionArc) {
    let stdout = stdout();
    writeln!(&mut stdout.lock(), "Starting receive loop!").unwrap();
    let mut running = RUNNING.load(Ordering::Relaxed);
    let mut ms = 0;
    loop_at!(10, ms, {
        running = RUNNING.load(Ordering::Relaxed);
        if !running {
            break;
        }
        for package in connection.collect_packages() {
            let data_slice = package.data.as_slice();
            let message = String::from_utf8_lossy(data_slice).clone();
            writeln!(&mut stdout.lock(), "{}", message).unwrap();
        }
    });
    // Exit the thread
}
