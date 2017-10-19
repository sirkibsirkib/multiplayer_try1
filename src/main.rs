
use std::sync::{Arc, Mutex, Condvar};
use std::thread;

#[macro_use]
extern crate serde_derive;

mod engine;
mod network;
mod world;

use network::{MsgQueue, MsgQueueStruct};

fn main() {
    let mut incoming : MsgQueue = Arc::new(MsgQueueStruct::new());
    let mut outgoing : MsgQueue = Arc::new(MsgQueueStruct::new());
    let mut incoming_clone = incoming.clone();
    let mut outgoing_clone = outgoing.clone();

    let args : Vec<_> = std::env::args().collect();
    if args.len() == 2 {
        let ri = network::ClientsideInformant::new(incoming, outgoing);
        thread::spawn(move || {
            network::server_entrypoint(&args[1], incoming_clone, outgoing_clone);
        });
        engine::game_loop(ri, 1);
    } else if args.len() == 3 {
        let ri = network::ClientsideInformant::new(incoming, outgoing);
        thread::spawn(move || {
            network::client_connect(&args[1], &args[2], incoming_clone, outgoing_clone);
        });
        engine::game_loop(ri, 2);
    } else {
        println!("Client mode [host] [port]\nServer mode [port]");
    }
}
