
use std;
use std::net::TcpStream;
use std::thread;
use std::io::Read;

use super::{TaggedStream, MsgQueue, Diff};

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
                super::communicate_out(t_clone, outgoing);
            });
            client_communicate_in(t, incoming);
        },
        Err(_) => {
            println!("No response.");
            panic!();
        }
    }
}


fn client_communicate_in(mut tagged_stream : TaggedStream, incoming : MsgQueue) {
    let mut buf = [0; 256];
    loop {
        //blocks until something is there
        match tagged_stream.stream.read(&mut buf) {
            Ok(bytes) => {
                let d : Diff = super::parse_diff(std::str::from_utf8(&buf[..bytes]).unwrap());
                println!("incoming {:?}", &d);
                incoming.v.lock().unwrap().push(d);
            },
            Err(msg) => match msg.kind() {
                std::io::ErrorKind::ConnectionReset => {println!("Connection reset!"); return;},
                x => println!("unexpected kind `{:?}`", x),
            },
        }
    }
}
