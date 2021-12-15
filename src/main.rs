mod machine;
use flexi_logger::{Duplicate, FileSpec, Logger};
use machine::Machine;
use std::env;
use std::fs;

fn main() {
    let mut args = env::args();
    args.next();
    let log_level = args.next().unwrap();
    Logger::try_with_str(log_level)
        .unwrap()
        .log_to_file(FileSpec::default().suppress_timestamp())
        .duplicate_to_stderr(Duplicate::Warn)
        .start()
        .unwrap();
    for arg in args {
        let bytes = match fs::read(&arg) {
            Ok(b) => b,
            Err(e) => {
                log::warn!("Cant open {}: {}", arg, e);
                continue;
            }
        };
        log::info!("Executing {}", arg);
        let mut prog = Vec::new();
        for bc in bytes.chunks(2) {
            assert_eq!(bc.len(), 2);
            let word = bc[0] as u16 + ((bc[1] as u16) << 8);
            prog.push(word);
        }
        let mut m = Machine::new(prog);
        m.run();
    }
    // let prog = vec![9, 32768, 32769, 60, 19, 32768];
    // let hello_world = vec![
    //     19, 72, 19, 101, 19, 108, 19, 108, 19, 111, 19, 32, 19, 87, 19, 111, 19, 114, 19, 108, 19,
    //     100, 19, 33, 19, 10, 0,
    // ];
}
