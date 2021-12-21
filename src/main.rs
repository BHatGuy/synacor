mod machine;
mod debugger;
use machine::Machine;
use debugger::Debugger;
use std::env;
use std::fs;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc;
use std::thread;

const SOCKET: &str = "/tmp/synacor.sock";

fn handle_client(stream: UnixStream, tx: mpsc::Sender<String>, rx: mpsc::Receiver<String>) {
    let mut stream_out = stream.try_clone().unwrap();
    let stream = BufReader::new(stream);
    write!(stream_out, "> ");
    for line in stream.lines() {
        let line = line.unwrap();
        tx.send(line).unwrap();
        let answer = rx.recv().unwrap();
        writeln!(stream_out, "{}", answer).unwrap();
        write!(stream_out, "> ").unwrap();
    }
}

fn cleanup() {
    fs::remove_file(SOCKET).unwrap();
}


fn main() {
    ctrlc::set_handler(move || {
        cleanup();
        std::process::exit(1)
    })
    .expect("Error setting Ctrl-C handler");

    let listener = UnixListener::bind(SOCKET).unwrap();
    let (tx, r) = mpsc::channel();
    let (s, rx) = mpsc::channel();
    println!("Waiting for debugger...");
    match listener.accept() {
        Ok((stream, _)) => {
            thread::spawn(move || handle_client(stream, s, r));
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    };

    let bytes = match fs::read(env::args().skip(1).next().unwrap()) {
        Ok(b) => b,
        Err(_) => {
            return
        }
    };
    let mut m = Machine::new(bytes);
    let mut debugger = Debugger::new(tx, rx);

    while !m.halted() {
        debugger.debugger_step(&mut m);
        m.step();
    }


    cleanup();
}
