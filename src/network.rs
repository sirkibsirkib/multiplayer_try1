
use std;
use std::io::{stdin,stdout};
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::io::prelude::*;
use std::sync::{Arc, Mutex, Condvar};

use engine::Point;

extern crate serde;
extern crate serde_json;



use engine::EntityID;

//represents a difference between the LOCAL and REMOTE
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Diff {
    Creation(EntityID, Point),
    Movement(EntityID, Point),
}

pub trait RemoteInformant {
    fn update(&mut self, diff : Diff);
    fn drain(&mut self) -> Vec<Diff>;
}

struct TaggedStream {
    tag : u32,
    stream : TcpStream,
}

//no need to inform anyone lel
// struct SingleplayerInformant {}
// impl RemoteInformant for SingleplayerInformant {
//     fn update(diff : Diff) {} //nothing to do here rofl
// }

pub struct ClientsideInformant {
    incoming : Arc<MsgQueueStruct>,
    outgoing : Arc<MsgQueueStruct>,
}



impl ClientsideInformant {
    pub fn new(incoming : Arc<MsgQueueStruct>, outgoing : Arc<MsgQueueStruct>) -> ClientsideInformant {
        ClientsideInformant {
            incoming : incoming,
            outgoing : outgoing,
        }
    }
}
impl RemoteInformant for ClientsideInformant {
    fn update(&mut self, diff : Diff) {
        let mut x = self.outgoing.v.lock().unwrap();
        x.push(diff);
        println!("Adding diff to outgoing. outgoing has len {}", x.len());
        self.outgoing.c.notify_all();
    }
    fn drain(&mut self) -> Vec<Diff> {
        let mut x = self.incoming.v.lock().unwrap();
        let y = x.drain(..).collect();
        y
    }
}
pub struct MsgQueueStruct {
    pub v : Mutex<Vec<Diff>>,
    pub c : Condvar,
}
impl MsgQueueStruct {
    pub fn new() -> MsgQueueStruct {
        MsgQueueStruct {
            v : Mutex::new(Vec::new()),
            c : Condvar::new(),
        }
    }
}

pub type MsgQueue = Arc<MsgQueueStruct>;


pub fn client_connect(host : &str, port : &str, incoming : MsgQueue, outgoing : MsgQueue) {
    match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(stream) => {
            stream.set_read_timeout(None).is_ok();
            let t = TaggedStream {
                tag : 1337, //clients only have one stream anyway
                stream : stream,
            };
            let t_clone = TaggedStream {
                tag : 1337, //clients only have one stream anyway
                stream : t.stream.try_clone().unwrap(),
            };
            println!("Client connected with tag {}", t.tag);
            thread::spawn(move || {
                communicate_out(t_clone, outgoing);
            });
            communicate_in(t, incoming, None);
        },
        Err(_) => {
            println!("No response.");
        }
    }
}

pub fn server_entrypoint(port : &str, incoming : MsgQueue, outgoing : MsgQueue) {
    println!("Server branch begin");
    let streams : Arc<Mutex<Vec<TaggedStream>>>
        = Arc::new(Mutex::new(Vec::new()));
    let bounced_diffs : Arc<Mutex<Vec<(Diff, u32)>>>
        = Arc::new(Mutex::new(Vec::new()));

    let streams_clone = streams.clone();
    let bounced_diffs_clone = bounced_diffs.clone();
    thread::spawn(move || {
        serve(streams_clone, outgoing, bounced_diffs_clone);
    });
    listen_for_new_clients(port, streams, incoming, bounced_diffs);
}

fn listen_for_new_clients(port : &str, streams : Arc<Mutex<Vec<TaggedStream>>>, incoming : MsgQueue, bounced_diffs : Arc<Mutex<Vec<(Diff, u32)>>>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .expect("Couldn't listen to that port!");

    let mut next_tag : u32 = 0;

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
            communicate_in(t_stream_clone, incoming_clone, Some(clone_bounced_diffs));
        });
        streams.lock().unwrap().push(t_stream);
        println!("Added client successfully");
        next_tag += 1;
    }
}

fn serve(tagged_streams : Arc<Mutex<Vec<TaggedStream>>>, outgoing : MsgQueue, bounced_diffs : Arc<Mutex<Vec<(Diff, u32)>>>) {
    //TODO bounce client diffs immediately in different method
    //TODO sleep on conditions for incoming and outgoing
    println!("Server serving outgoing");
    let mut todo : Vec<(String, u32)> = Vec::new();
    loop {
        {
            //grab outgoing lock. serialize and consume my own diffs
            let mut my_outgoing = outgoing.v.lock().unwrap();
            for o in my_outgoing.drain(..) {
                println!("Serv-sending {:?}", &o);
                let serialized = serde_json::to_string(&o).unwrap();
                todo.push((serialized, 9999)); //everybody gets this one. i generated it myself
            }
            //release outgoing lock
        }
        {
            //grab bounced_diffs lock. serialize and consume bounced diffs
            let mut my_diffs = bounced_diffs.lock().unwrap();
            for c in my_diffs.drain(..) {
                println!("Serv-bouncing {:?}", &c);
                let serialized = serde_json::to_string(&c.1).unwrap();
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
                    }
                }
            }
            //release streams lock
        }
    }
}

fn communicate_in(mut tagged_stream : TaggedStream, incoming : MsgQueue, maybe_diffs : Option<Arc<Mutex<Vec<(Diff, u32)>>>>) {
    let mut buf = [0; 256];
    loop {
        match tagged_stream.stream.read(&mut buf) {
            Ok(bytes) => {
                let d : Diff = parse_diff(std::str::from_utf8(&buf[..bytes]).unwrap());
                println!("incoming {:?}", &d);
                incoming.v.lock().unwrap().push(d);
                if let Some(ref x) = maybe_diffs {
                    println!("marking for bounce {:?}", &d);
                    let z = (d, tagged_stream.tag);
                    x.lock().unwrap().push(z);
                }
            },
            Err(msg) => match msg.kind() {
                std::io::ErrorKind::ConnectionReset => {println!("Connection reset!"); return;},
                x => println!("unexpected kind `{:?}`", x),
            },
        }
    }
}

fn parse_diff (s : &str) -> Diff {
    let x : Diff = serde_json::from_str(s).unwrap();
    x
}


fn communicate_out(mut tagged_stream : TaggedStream, outgoing : MsgQueue) {
    // let second = std::time::Duration::from_millis(1000);
    loop {
        let mut vec = outgoing.v.lock().unwrap();
        while vec.is_empty() {
            vec = outgoing.c.wait(vec).unwrap()
        }
        if let Some(diff) = vec.pop() {
            println!("sending out {:?}", &diff);
            let serialized = serde_json::to_string(&diff).unwrap();
            tagged_stream.stream.write(serialized.as_bytes()).is_ok();
        }
    }
    //
    // println!("Waiting for user input lines:");
    // let mut s;
    // loop {
    //     s = String::new();
    //     let _ = stdout().flush();
    //     stdin().read_line(&mut s).expect("Bad user input");
    //     s.trim();
    //     match tagged_stream.stream.write(s.as_bytes()) {
    //         Err(msg) => if msg.kind() == std::io::ErrorKind::ConnectionReset {return;},
    //         _ => (),
    //     }
    // }
}
