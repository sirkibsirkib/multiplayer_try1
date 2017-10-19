
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, Condvar};

mod client;
mod server;

use world::Point;

extern crate serde;
extern crate serde_json;

type StreamID = u32;
pub type MsgQueue = Arc<MsgQueueStruct>;

use world::EntityID;

//represents a difference between the LOCAL and REMOTE

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
enum NetworkMessage {
    //all the types of communications between client and server
    Diff,
    MetaMessage,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Diff {
    //normal operation updates of game state being flooded from one machine to all the rest
    Creation(EntityID, Point),
    Movement(EntityID, Point),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MetaMessage {
    //message requests for information. server will not flood if it can respond directly
    RequestEntity(EntityID),
    ProvideEntity(EntityID, Point),
}

pub trait RemoteInformant {
    fn update_all(&mut self, diffs : Vec<Diff>);
    fn update(&mut self, diff : Diff);
    fn drain(&mut self) -> Vec<Diff>;
}

pub struct TaggedStream {
    tag : StreamID,
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
    fn update_all(&mut self, diffs : Vec<Diff>) {
        let mut x = self.outgoing.v.lock().unwrap();
        for diff in diffs {
            x.push(diff);
        }
        self.outgoing.c.notify_all();
    }
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

//Multiple threads can share an Arc<SharedQueue<T>>. consumer threads will wait() when draining
//producer threads will notify when producing
struct SharedQueue<T> {
    pub queue : Mutex<Vec<T>>,
    pub cond : Condvar,
}

// impl SharedQueue {
// }

impl<T> SharedQueue<T> {
    fn new() -> Self {
        SharedQueue {
            queue : Mutex::new(Vec::new()),
            cond : Condvar::new(),
        }
    }

    fn lock_push_notify(&self, t : T) {
        let mut locked_queue = self.queue.lock().unwrap();
        locked_queue.push(t);
        self.cond.notify_all();
    }

    fn wait_until_nonempty_drain(&self) -> Vec<T>{
        let mut locked_queue = self.queue.lock().unwrap();
        while locked_queue.is_empty() {
            locked_queue = self.cond.wait(locked_queue).unwrap();
        }
        let t = locked_queue.drain(..).collect();
        t
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


pub fn client_entrypoint(host : &str, port : &str, incoming : MsgQueue, outgoing : MsgQueue) {
    client::client_connect(host, port, incoming, outgoing);
}

pub fn server_entrypoint(port : &str, incoming : MsgQueue, outgoing : MsgQueue){
    server::server_branch(port, incoming, outgoing);
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
}
