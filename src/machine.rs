use std::collections::HashMap;
enum Operation {
    Halt(),
    Set,
    Push,
    Pop,
    Eq,
    Gt,
    Jmp,
    Jt,
    Jf,
    Add(u16, u16, u16),
    Mult,
    Mod,
    And,
    Or,
    Not,
    Rmem,
    Wmem,
    Call,
    Ret,
    Out(u16),
    In,
    Noop(),
}

#[derive(Debug)]
pub struct Machine {
    memory: HashMap<u16, u16>,
    regfile: [u16; 8],
    stack: Vec<u16>,
    pc: u16,
    halted: bool
}

impl Operation {
    fn len(&self) -> u16 {
        match self {
            Operation::Halt() => 1,
            Operation::Add(_, _, _) => 4,
            Operation::Out(_) => 2,
            Operation::Noop() => 1,
            _ => panic!("not implemented"),
        }
    }
}

impl Machine {
    pub fn new(prog: Vec<u16>) -> Self {
        let mut mem = HashMap::new();
        for (i, b) in prog.iter().enumerate() {
            let i = i as u16;
            mem.insert(i, *b);
        }
        Machine {
            memory: mem,
            regfile: [0u16; 8],
            stack: Vec::new(),
            pc: 0,
            halted: false
        }
    }

    fn fetch(&self) -> Operation {
        match self.memory.get(&self.pc).unwrap_or(&0) {
            0 => Operation::Halt(),
            9 => Operation::Add(
                self.memory[&(self.pc + 1)],
                self.memory[&(self.pc + 2)],
                self.memory[&(self.pc + 3)],
            ),
            19 => Operation::Out(self.memory[&(self.pc + 1)]),
            21 => Operation::Noop(),
            _ => panic!("invalid Opcode"),
        }
    }


    fn execute(&mut self, op: &Operation){
        match op {
            Operation::Halt() => self.halted = true,
            Operation::Out(a) => self.out(*a),
            Operation::Add(a, b, c) => self.add(*a, *b, *c),
            _ => panic!("ex not implemented"),
        }
    }
    
    pub fn run(&mut self) {
        while !self.halted{
            self.step();
        }
    }

    pub fn step(&mut self) {
        if self.halted {
            return;
        }
        let op = self.fetch();
        self.execute(&op);
        self.pc += op.len();
    }

    fn get_val(&self, a: u16) -> u16 {
        if a < 0x8000 {
            a
        } else {
            self.regfile[self.get_reg(a)]
        }
    }

    fn get_reg(&self, a: u16) -> usize {
        if a < 0x8000 {
            panic!("invalid register");
        }
        (a & 0x7fff) as usize
    }

    fn add(&mut self, a: u16, b: u16, c: u16) {
        self.regfile[self.get_reg(a)] = self.get_val(b) + self.get_val(c);
    }

    fn out(&self, a: u16) {
        let a: u8 = self.get_val(a).try_into().expect("Cant print that!");
        let c = (a) as char;
        print!("{}", c);
    }
}
