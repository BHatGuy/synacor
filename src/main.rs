mod machine;
use machine::Machine;
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
    for line in stream.lines() {
        let line = line.unwrap();
        tx.send(line).unwrap();
        let answer = rx.recv().unwrap();
        writeln!(stream_out, "{}", answer).unwrap();
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

    for arg in env::args().skip(1) {
        let bytes = match fs::read(&arg) {
            Ok(b) => b,
            Err(_) => {
                continue;
            }
        };
        let mut m = Machine::new(bytes);
        while !m.halted() {
            if let Ok(command) = rx.try_recv() {
                let answer;
                match command.as_str() {
                    "dump" => {
                        let mut file = fs::File::create("state.bin").unwrap();
                        let bytes = m.dump();
                        file.write_all(&bytes).unwrap();
                        answer = "dumped to state.bin".to_owned();
                    }
                    _ => answer = "Unknown command!".to_owned(),
                }
                tx.send(answer).unwrap();
            }
            m.step();
        }
    }
    cleanup();
}
