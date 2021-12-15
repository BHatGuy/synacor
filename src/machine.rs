use std::collections::HashMap;

#[derive(Debug)]
enum Operation {
    Halt(),
    Set,
    Push,
    Pop,
    Eq,
    Gt,
    Jmp(u16),
    Jt(u16, u16),
    Jf(u16, u16),
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
    halted: bool,
}

impl Operation {
    fn len(&self) -> u16 {
        match self {
            Operation::Halt() => 1,
            Operation::Jmp(_) => 2,
            Operation::Jt(_, _) => 3,
            Operation::Jf(_, _) => 3,
            Operation::Add(_, _, _) => 4,
            Operation::Out(_) => 2,
            Operation::Noop() => 1,
            _ => panic!("{:?} not implemented", self),
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
            halted: false,
        }
    }

    fn fetch(&self) -> Operation {
        let code = self.memory.get(&self.pc).unwrap_or(&0);
        match code {
            0 => Operation::Halt(),
            6 => Operation::Jmp(self.memory[&(self.pc + 1)]),
            7 => Operation::Jt(self.memory[&(self.pc + 1)], self.memory[&(self.pc + 2)]),
            8 => Operation::Jf(self.memory[&(self.pc + 1)], self.memory[&(self.pc + 2)]),
            9 => Operation::Add(
                self.memory[&(self.pc + 1)],
                self.memory[&(self.pc + 2)],
                self.memory[&(self.pc + 3)],
            ),
            19 => Operation::Out(self.memory[&(self.pc + 1)]),
            21 => Operation::Noop(),
            _ => panic!("invalid opcode ({})", code),
        }
    }

    fn execute(&mut self, op: &Operation) {
        match op {
            Operation::Halt() => self.halt(),
            Operation::Jmp(a) => self.jump(*a),
            Operation::Jt(a, b) => self.jump_true(*a, *b),
            Operation::Jf(a, b) => self.jump_false(*a, *b),
            Operation::Add(a, b, c) => self.add(*a, *b, *c),
            Operation::Out(a) => self.out(*a),
            Operation::Noop() => self.noop(),
            _ => panic!("{:?} not implemented", op),
        }
    }
    pub fn run(&mut self) {
        while !self.halted {
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
            panic!("invalid register ({})", a);
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

    fn jump(&mut self, a: u16) {
        let target = self.get_val(a);
        self.pc = target - Operation::Jmp(0).len(); // Account for pc inc
    }

    fn jump_true(&mut self, a: u16, b: u16) {
        if self.get_val(a) != 0 {
            let target = self.get_val(b);
            self.pc = target - Operation::Jt(0, 0).len(); // Account for pc inc
        }
    }

    fn jump_false(&mut self, a: u16, b: u16) {
        if self.get_val(a) == 0 {
            let target = self.get_val(b);
            self.pc = target - Operation::Jf(0, 0).len(); // Account for pc inc
        }
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    fn noop(&self) {}
}
