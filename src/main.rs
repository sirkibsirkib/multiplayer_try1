use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::io::prelude::*;



fn main() {
    let args : Vec<_> = std::env::args().collect();
    if args.len() > 1 && args[1] == "server" {
        println!("SERVER");
        server_loop();
    } else {
        match TcpStream::connect("127.0.0.1:8080") {
            Ok(stream) => {
                // stream.set_nonblocking(true).is_ok();
                let second = std::time::Duration::from_millis(1000);
                stream.set_read_timeout(None).is_ok();
                communicate(stream);
            },
            Err(_) => {
                println!("No response.");
            }
        }
    }
}


fn server_loop() {
    let host = "127.0.0.1";
    let port = 8080;
    let second = std::time::Duration::from_millis(1000);

    let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
    println!("listening started, ready to accept");
    for stream in listener.incoming() {
        thread::spawn(move || {
            let stream = stream.unwrap();
            // stream.set_nonblocking(true).is_ok();
            stream.set_read_timeout(None).is_ok();
            communicate(stream);
        });
    }
}

fn communicate(mut stream : TcpStream) {

    let clone = stream.try_clone().unwrap();
    thread::spawn(move || {
        communicate_out(clone);
    });



    let mut buf = [0; 100];
    let second = std::time::Duration::from_millis(1000);
    loop {
        // stream.write("Hello".as_bytes()).is_ok();

        // thread::sleep(second);
        // println!("??");
        match stream.read(&mut buf) {
            Ok(bytes) => println!("read {:?} ie `{}`", bytes, std::str::from_utf8(&buf[..bytes]).unwrap()),
            Err(msg) => match msg.kind() {
                std::io::ErrorKind::ConnectionReset => {println!("Connection reset!"); return;},
                x => println!("unexpected kind `{:?}`", x),
            },
        }
    }
}
use std::io::{stdin,stdout};

fn communicate_out(mut stream : TcpStream) {
    println!("Waiting for user input lines:");
    let mut s;
    loop {
        s = String::new();
        let _ = stdout().flush();
        stdin().read_line(&mut s).expect("Did not enter a correct string");
        s.trim();
        println!("WRITING");
        match stream.write(s.as_bytes()) {
            Err(msg) => if msg.kind() == std::io::ErrorKind::ConnectionReset {return;},
            _ => (),
        }
    }
}
