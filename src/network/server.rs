
use std;
use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

use super::{MsgQueue, TaggedStream, SharedQueue, Diff, StreamID};

pub fn server_branch(port : &str, incoming : MsgQueue, outgoing : MsgQueue) {
    println!("Server branch begin");
    let streams : Arc<Mutex<Vec<TaggedStream>>>
        = Arc::new(Mutex::new(Vec::new()));
    let bounced_diffs = Arc::new(
        SharedQueue::new()
    );

    let streams_clone = streams.clone();
    let bounced_diffs_clone = bounced_diffs.clone();
    let streams_clone2 = streams.clone();
    thread::spawn(move || {
        serve_own(streams_clone, outgoing);
    });
    thread::spawn(move || {
        serve_bounces(streams_clone2, bounced_diffs_clone);
    });
    listen_for_new_clients(port, streams, incoming, bounced_diffs);
}

fn listen_for_new_clients(port : &str, streams : Arc<Mutex<Vec<TaggedStream>>>, incoming : MsgQueue, bounced_diffs : Arc<SharedQueue<(Diff,StreamID)>>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .expect("Couldn't listen to that port!");

    let mut next_tag : StreamID = 0;

    println!("Server listening for clients");
    for s in listener.incoming() {
        let t_stream = TaggedStream {
            tag : next_tag,
            stream : s.unwrap(),
        };
        let t_stream_clone = TaggedStream {
            tag : next_tag,
            stream : t_stream.stream.try_clone().unwrap(),
        };
        println!("Adding new client with port ID-tag {}", next_tag);
        let incoming_clone = incoming.clone();
        let clone_bounced_diffs = bounced_diffs.clone();
        thread::spawn(move || {
            println!("creating dedicated incoming listener for client");
            server_communicate_in(t_stream_clone, incoming_clone, clone_bounced_diffs);
        });
        streams.lock().unwrap().push(t_stream);
        println!("Added client successfully");
        next_tag += 1;
    }
}

fn serve_own(tagged_streams : Arc<Mutex<Vec<TaggedStream>>>, outgoing : MsgQueue) {
    println!("Server serving own");
    let mut todo : Vec<(String, StreamID)> = Vec::new();
    loop {
        println!("serve loop");
        {
            //wait until there is something outgoing
            let mut out_v = outgoing.v.lock().unwrap();
            while out_v.is_empty() {
                out_v = outgoing.c.wait(out_v).unwrap();
            }
            for o in out_v.drain(..) {
                println!("Serv-sending {:?}", &o);
                let serialized = super::serde_json::to_string(&o).unwrap();
                todo.push((serialized, 9999)); //everybody gets this one. i generated it myself
            }
            //release outgoing lock
        }

        if ! todo.is_empty() {
            // grab streams lock. write serialized diffs
            let mut t_streams = tagged_streams.lock().unwrap();
            for t_stream in t_streams.iter_mut() {
                for t in todo.drain(..) {
                    if t.1 != t_stream.tag {
                        t_stream.stream.write(&t.0.as_bytes()).is_ok();
                        //TODO might not be OK. handle disconnection
                    }
                }
            }
            //release streams lock
        }
    }
}

fn serve_bounces(tagged_streams : Arc<Mutex<Vec<TaggedStream>>>, bounced_diffs : Arc<SharedQueue<(Diff,StreamID)>>) {
    println!("Server serving bounces");
    let mut todo : Vec<(String, u32)> = Vec::new();
    loop {
        println!("serve loop");
        {
            //grab bounced_diffs lock. serialize and consume bounced diffs
            for c in bounced_diffs.wait_until_nonempty_drain() {
                println!("Serv-bouncing {:?}", &c);
                let serialized = super::serde_json::to_string(&c.1).unwrap();
                todo.push((serialized, c.1)); // client with id c.1 doesnt get this one
            }
            //release bounced_diffs lock
        }

        if ! todo.is_empty() {
            // grab streams lock. write serialized diffs
            let mut t_streams = tagged_streams.lock().unwrap();
            for t_stream in t_streams.iter_mut() {
                for t in todo.drain(..) {
                    if t.1 != t_stream.tag {
                        t_stream.stream.write(&t.0.as_bytes()).is_ok();
                        //TODO might not be OK. handle disconnection
                    }
                }
            }
            //release streams lock
        }
    }
}

fn server_communicate_in(mut tagged_stream : TaggedStream, incoming : MsgQueue, maybe_diffs : Arc<SharedQueue<(Diff,StreamID)>>) {
    let mut buf = [0; 256];
    loop {
        //blocks until something is there
        match tagged_stream.stream.read(&mut buf) {
            Ok(bytes) => {
                let d : Diff = super::parse_diff(std::str::from_utf8(&buf[..bytes]).unwrap());
                println!("incoming {:?}", &d);
                incoming.v.lock().unwrap().push(d);
                let z = (d, tagged_stream.tag);
                maybe_diffs.lock_push_notify(z);
            },
            Err(msg) => match msg.kind() {
                std::io::ErrorKind::ConnectionReset => {println!("Connection reset!"); return;},
                x => println!("unexpected kind `{:?}`", x),
            },
        }
    }
}
