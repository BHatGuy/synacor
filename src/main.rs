mod machine;
use machine::Machine;

fn main() {
    let prog = vec![9, 32768, 32769, 60, 19, 32768];
    let hello_world = vec![
        19, 72, 19, 101, 19, 108, 19, 108, 19, 111, 19, 32, 19, 87, 19, 111, 19, 114, 19, 108, 19,
        100, 19, 33, 19, 10, 0,
    ];
    let mut m = Machine::new(prog);

    m.run();
}
