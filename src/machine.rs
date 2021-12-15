use std::collections::HashMap;

#[derive(Debug)]
enum Operation {
    Halt(),
    Set(u16, u16),
    Push(u16),
    Pop(u16),
    Eq(u16, u16, u16),
    Gt(u16, u16, u16),
    Jmp(u16),
    Jt(u16, u16),
    Jf(u16, u16),
    Add(u16, u16, u16),
    Mult(u16, u16, u16),
    Mod(u16, u16, u16),
    And(u16, u16, u16),
    Or(u16, u16, u16),
    Not(u16, u16),
    Rmem(u16, u16),
    Wmem(u16, u16),
    Call(u16),
    Ret(),
    Out(u16),
    In(u16),
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
            Operation::Set(_, _) => 3,
            Operation::Push(_) => 2,
            Operation::Pop(_) => 2,
            Operation::Eq(_, _, _) => 4,
            Operation::Gt(_, _, _) => 4,
            Operation::Jmp(_) => 2,
            Operation::Jt(_, _) => 3,
            Operation::Jf(_, _) => 3,
            Operation::Add(_, _, _) => 4,
            Operation::Mult(_, _, _) => 4,
            Operation::Mod(_, _, _) => 4,
            Operation::And(_, _, _) => 4,
            Operation::Or(_, _, _) => 4,
            Operation::Not(_, _) => 3,
            Operation::Rmem(_, _) => 3,
            Operation::Wmem(_, _) => 3,
            Operation::Call(_) => 2,
            Operation::Ret() => 1,
            Operation::Out(_) => 2,
            Operation::In(_) => 2,
            Operation::Noop() => 1,
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
        let a = self.memory[&(self.pc + 1)];
        let b = self.memory[&(self.pc + 2)];
        let c = self.memory[&(self.pc + 3)];
        match code {
            0 => Operation::Halt(),
            1 => Operation::Set(a, b),
            2 => Operation::Push(a),
            3 => Operation::Pop(a),
            4 => Operation::Eq(a, b, c),
            5 => Operation::Gt(a, b, c),
            6 => Operation::Jmp(a),
            7 => Operation::Jt(a, b),
            8 => Operation::Jf(a, b),
            9 => Operation::Add(a, b, c),
            10 => Operation::Mult(a, b, c),
            11 => Operation::Mod(a, b, c),
            12 => Operation::And(a, b, c),
            13 => Operation::Or(a, b, c),
            14 => Operation::Not(a, b),
            15 => Operation::Rmem(a, b),
            16 => Operation::Wmem(a, b),
            17 => Operation::Call(a),
            18 => Operation::Ret(),
            19 => Operation::Out(a),
            20 => Operation::In(a),
            21 => Operation::Noop(),
            _ => panic!("invalid opcode ({})", code),
        }
    }

    fn execute(&mut self, op: &Operation) {
        match op {
            Operation::Halt() => self.halt(),
            Operation::Set(a, b) => self.set(*a, *b),
            Operation::Push(a) => self.push(*a),
            Operation::Pop(a) => self.pop(*a),
            Operation::Eq(a, b, c) => self.eq(*a, *b, *c),
            Operation::Gt(a, b, c) => self.gt(*a, *b, *c),
            Operation::Jmp(a) => self.jump(*a),
            Operation::Jt(a, b) => self.jump_true(*a, *b),
            Operation::Jf(a, b) => self.jump_false(*a, *b),
            Operation::Add(a, b, c) => self.add(*a, *b, *c),
            Operation::And(a, b, c) => self.and(*a, *b, *c),
            Operation::Or(a, b, c) => self.or(*a, *b, *c),
            Operation::Not(a, b) => self.not(*a, *b),
            Operation::Call(a) => self.call(*a),
            Operation::Ret() => self.ret(),
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
        self.pc += op.len();
        self.execute(&op);
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

    fn set(&mut self, a: u16, b: u16) {
        self.regfile[self.get_reg(a)] = self.get_val(b);
    }

    fn push(&mut self, a: u16) {
        self.stack.push(self.get_val(a));
    }

    fn pop(&mut self, a: u16) {
        self.regfile[self.get_reg(a)] = self.stack.pop().unwrap();
    }

    fn eq(&mut self, a: u16, b: u16, c: u16) {
        self.regfile[self.get_reg(a)] = if self.get_val(b) == self.get_val(c) {
            1
        } else {
            0
        }
    }

    fn gt(&mut self, a: u16, b: u16, c: u16) {
        self.regfile[self.get_reg(a)] = if self.get_val(b) > self.get_val(c) {
            1
        } else {
            0
        }
    }

    fn add(&mut self, a: u16, b: u16, c: u16) {
        self.regfile[self.get_reg(a)] = (self.get_val(b) + self.get_val(c)) % 0x8000;
    }

    fn and(&mut self, a: u16, b: u16, c: u16) {
        self.regfile[self.get_reg(a)] = self.get_val(b) & self.get_val(c);
    }

    fn or(&mut self, a: u16, b: u16, c: u16) {
        self.regfile[self.get_reg(a)] = self.get_val(b) | self.get_val(c);
    }

    fn not(&mut self, a: u16, b: u16) {
        self.regfile[self.get_reg(a)] = (!self.get_val(b)) & 0x7FFF;
    }

    fn call(&mut self, a: u16) {
        self.push(self.pc);
        let target = self.get_val(a);
        self.jump(target)
    }

    fn ret(&mut self) {
        if let Some(target) = self.stack.pop() {
            self.jump(target);
        } else {
            self.halt();
        }
    }

    fn out(&self, a: u16) {
        let a: u8 = self.get_val(a).try_into().expect("Cant print that!");
        let c = (a) as char;
        print!("{}", c);
    }

    fn jump(&mut self, a: u16) {
        let target = self.get_val(a);
        self.pc = target;
    }

    fn jump_true(&mut self, a: u16, b: u16) {
        if self.get_val(a) != 0 {
            self.jump(b);
        }
    }

    fn jump_false(&mut self, a: u16, b: u16) {
        if self.get_val(a) == 0 {
            self.jump(b);
        }
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    fn noop(&self) {}
}
