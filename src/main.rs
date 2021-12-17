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

fn process_comand(cmd: String, m: &mut Machine, running: &mut bool) -> String {
    let command: Vec<&str> = cmd.split(" ").collect();
    // TODO break, single step, continue
    match command[0] {
        "dump" => {
            let mut file = fs::File::create("state.bin").unwrap();
            let bytes = m.dump();
            file.write_all(&bytes).unwrap();
            return format!("dumped to state.bin ({} bytes)", bytes.len());
        }
        "restore" => {
            let bytes = match fs::read("state.bin") {
                Ok(b) => b,
                Err(_) => {
                    return "Cant read state".to_owned();
                }
            };
            m.restore(&bytes);
            return format!("restored state.bin ({} bytes)", bytes.len());
        }
        "set" => {
            if command.len() == 3 {
                if let (Ok(idx), Ok(val)) = (command[1].parse(), command[2].parse()) {
                    m.set_reg(idx, val);
                    return format!("Set reg {} to {:#x}", idx, val);
                } else {
                    return format!("Invalid set command! {:?}", command);
                }
            } else {
                return format!("Invalid set command! {:?}", command);
            }
        }
        "get" => {
            if command.len() == 2 {
                if let Ok(idx) = command[1].parse() {
                    let val = m.get_register(idx);
                    return format!("reg[{}]={:#x}", idx, val);
                } else {
                    return format!("Invalid get command! {:?}", command);
                }
            } else {
                return format!("Invalid get command! {:?}", command);
            }
        }
        "stop" => {
            *running = false;
            return format!("stopped");
        }
        "c" => {
            *running = true;
            return format!("continued");
        }
        "" => {
            m.step();
            return format!("{:#x}: {:x?}", m.pc, m.fetch());
        }
        _ => return format!("Unknown command! {:?}", command),
    }
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
        let mut running = false;
        while !m.halted() {
            if running {
                while let Ok(command) = rx.try_recv() {
                    let answer = process_comand(command, &mut m, &mut running);
                    tx.send(answer).unwrap();
                }
                m.step();
            } else {
                if let Ok(command) = rx.recv() {
                    let answer = process_comand(command, &mut m, &mut running);
                    tx.send(answer).unwrap();
                }
            }
            
        }
    }
    cleanup();
}
