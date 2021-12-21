use crate::machine::Machine;
use std::fs;
use std::io::Write;
use std::sync::mpsc;

pub struct Debugger {
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>,
    running: bool,
}

impl Debugger {
    pub fn new(tx: mpsc::Sender<String>, rx: mpsc::Receiver<String>) -> Self {
        Self {
            tx,
            rx,
            running: false,
        }
    }

    pub fn debugger_step(&mut self, m: &mut Machine) {
        if self.running {
            while let Ok(command) = self.rx.try_recv() {
                self.process_comand(command, m);
            }
        } else {
            while !self.running {
                if let Ok(command) = self.rx.recv() {
                    let ss = self.process_comand(command, m);
                    if ss {
                        break;
                    }
                }
            }
        }
    }

    fn process_comand(&mut self, cmd: String, m: &mut Machine) -> bool {
        let command: Vec<&str> = cmd.split(" ").collect();
        let answer;
        let mut ss = false;
        match command[0] {
            "dump" => {
                let mut file = fs::File::create("state.bin").unwrap();
                let bytes = m.dump();
                file.write_all(&bytes).unwrap();
                answer = format!("dumped to state.bin ({} bytes)", bytes.len());
            }
            "restore" => {
                if let Ok(bytes) = fs::read("state.bin") {
                    m.restore(&bytes);
                    answer = format!("restored state.bin ({} bytes)", bytes.len());
                } else {
                    answer = "Cant read state".to_owned();
                }
            }
            "set" => {
                if command.len() == 3 {
                    if let (Ok(idx), Ok(val)) = (command[1].parse(), command[2].parse()) {
                        m.set_reg(idx, val);
                        answer = format!("Set reg {} to {:#x}", idx, val);
                    } else {
                        answer = format!("Invalid set command! {:?}", command);
                    }
                } else {
                    answer = format!("Invalid set command! {:?}", command);
                }
            }
            "get" => {
                if command.len() == 2 {
                    if let Ok(idx) = command[1].parse() {
                        let val = m.get_register(idx);
                        answer = format!("reg[{}]={:#x}", idx, val);
                    } else {
                        answer = format!("Invalid get command! {:?}", command);
                    }
                } else {
                    answer = format!("Invalid get command! {:?}", command);
                }
            }
            "stop" => {
                self.running = false;
                answer = format!("stopped");
            }
            "c" => {
                self.running = true;
                answer = format!("continued");
            }
            "" => {
                ss = true;
                answer = format!("{:#06x}: {}", m.pc, m.fetch());
            }
            "dis" => {
                if command.len() == 3 {
                    let mut pos = u16::from_str_radix(command[1], 16).unwrap();
                    let len = command[2].parse().unwrap();
                    let mut asm = String::new();
                    for _ in 0..len {
                        let op = m.fetch_at(pos);
                        asm += &format!("{:#06x}: {}\n", pos, op);
                        pos += op.len();
                    }
                    answer = asm;
                } else {
                    answer = format!("Invalid dis (dis start(hex) length(dec))");
                }
            }
            _ => answer = format!("Unknown command! {:?}", command),
        };
        self.tx.send(answer).unwrap();
        ss
    }
}
