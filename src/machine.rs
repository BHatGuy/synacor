use std::fmt;
use std::io::{self, Read, Write};
use Operation::*;

#[derive(Debug)]
pub enum Operation {
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
    pub memory: [u16; 0x8000],
    pub regfile: [u16; 8],
    pub stack: Vec<u16>,
    pub pc: u16,
    pub halted: bool,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Halt() => "halt",
            Set(_, _) => "set",
            Push(_) => "push",
            Pop(_) => "pop",
            Eq(_, _, _) => "eq",
            Gt(_, _, _) => "gt",
            Jmp(_) => "jmp",
            Jt(_, _) => "jt",
            Jf(_, _) => "jf",
            Add(_, _, _) => "add",
            Mult(_, _, _) => "mul",
            Mod(_, _, _) => "mod",
            And(_, _, _) => "and",
            Or(_, _, _) => "or",
            Not(_, _) => "not",
            Rmem(_, _) => "rmem",
            Wmem(_, _) => "wmem",
            Call(_) => "call",
            Ret() => "ret",
            Out(_) => "out",
            In(_) => "in",
            Noop() => "noop",
        };
        let (a, b, c) = match self {
            Operation::Halt() | Operation::Noop() | Operation::Ret() => (None, None, None),
            Operation::Push(a)
            | Operation::In(a)
            | Operation::Out(a)
            | Operation::Call(a)
            | Operation::Jmp(a)
            | Operation::Pop(a) => (Some(a), None, None),
            Operation::Set(a, b)
            | Operation::Wmem(a, b)
            | Operation::Rmem(a, b)
            | Operation::Not(a, b)
            | Operation::Jf(a, b)
            | Operation::Jt(a, b) => (Some(a), Some(b), None),
            Operation::Eq(a, b, c)
            | Operation::Gt(a, b, c)
            | Operation::Add(a, b, c)
            | Operation::Mult(a, b, c)
            | Operation::Mod(a, b, c)
            | Operation::And(a, b, c)
            | Operation::Or(a, b, c) => (Some(a), Some(b), Some(c)),
        };
        write!(f, "{:<4}", name)?;
        if let Some(a) = a {
            write!(f, " {:4x}", a)?;
        }
        if let Some(b) = b {
            write!(f, " {:4x}", b)?;
        }
        if let Some(c) = c {
            write!(f, " {:4x}", c)?;
        }
        Ok(())
    }
}

impl Operation {
    pub fn len(&self) -> u16 {
        match self {
            Operation::Halt() | Operation::Noop() | Operation::Ret() => 1,
            Operation::Push(_)
            | Operation::In(_)
            | Operation::Out(_)
            | Operation::Call(_)
            | Operation::Jmp(_)
            | Operation::Pop(_) => 2,
            Operation::Set(_, _)
            | Operation::Wmem(_, _)
            | Operation::Rmem(_, _)
            | Operation::Not(_, _)
            | Operation::Jf(_, _)
            | Operation::Jt(_, _) => 3,
            Operation::Eq(_, _, _)
            | Operation::Gt(_, _, _)
            | Operation::Add(_, _, _)
            | Operation::Mult(_, _, _)
            | Operation::Mod(_, _, _)
            | Operation::And(_, _, _)
            | Operation::Or(_, _, _) => 4,
        }
    }
}

fn assemble_word(bc: &[u8]) -> u16 {
    assert_eq!(bc.len(), 2);
    bc[0] as u16 + ((bc[1] as u16) << 8)
}

impl Machine {
    pub fn new(prog: Vec<u8>) -> Self {
        let mut mem = [0u16; 0x8000];
        for (i, bc) in prog.chunks(2).enumerate() {
            let word = assemble_word(bc);
            mem[i as usize] = word;
        }
        Machine {
            memory: mem,
            regfile: [0u16; 8],
            stack: Vec::new(),
            pc: 0,
            halted: false,
        }
    }

    pub fn dump(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let b = self.pc.to_le_bytes();
        bytes.push(b[0]);
        bytes.push(b[1]);

        for word in self.regfile {
            let b = word.to_le_bytes();
            bytes.push(b[0]);
            bytes.push(b[1]);
        }
        for word in self.memory {
            let b = word.to_le_bytes();
            bytes.push(b[0]);
            bytes.push(b[1]);
        }

        for word in self.stack.iter() {
            let b = word.to_le_bytes();
            bytes.push(b[0]);
            bytes.push(b[1]);
        }
        bytes
    }

    pub fn restore(&mut self, bytes: &[u8]) {
        self.pc = assemble_word(&bytes[0..2]);
        let bytes = &bytes[2..];

        for (word, bc) in self.regfile.iter_mut().zip(bytes.chunks(2)) {
            *word = assemble_word(bc);
        }
        let bytes = &bytes[self.regfile.len() * 2..];

        for (word, bc) in self.memory.iter_mut().zip(bytes.chunks(2)) {
            *word = assemble_word(bc);
        }
        let bytes = &bytes[self.memory.len() * 2..];

        self.stack = Vec::new();
        for bc in bytes.chunks(2) {
            self.stack.push(assemble_word(bc));
        }
    }

    pub fn set_reg(&mut self, idx: usize, val: u16) {
        self.regfile[idx] = val;
    }

    pub fn get_register(&mut self, idx: usize) -> u16 {
        self.regfile[idx]
    }

    pub fn fetch(&self) -> Operation {
        self.fetch_at(self.pc)
    }

    pub fn fetch_at(&self, pos : u16) -> Operation {
        let code = self.memory[pos as usize];
        let a = self.memory[(pos + 1) as usize];
        let b = self.memory[(pos + 2) as usize];
        let c = self.memory[(pos + 3) as usize];
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
            Operation::Mult(a, b, c) => self.mult(*a, *b, *c),
            Operation::Mod(a, b, c) => self.modulo(*a, *b, *c),
            Operation::And(a, b, c) => self.and(*a, *b, *c),
            Operation::Or(a, b, c) => self.or(*a, *b, *c),
            Operation::Not(a, b) => self.not(*a, *b),
            Operation::Rmem(a, b) => self.rmem(*a, *b),
            Operation::Wmem(a, b) => self.wmem(*a, *b),
            Operation::Call(a) => self.call(*a),
            Operation::Ret() => self.ret(),
            Operation::Out(a) => self.out(*a),
            Operation::In(a) => self.inp(*a),
            Operation::Noop() => self.noop(),
        }
    }
    pub fn halted(&self) -> bool {
        self.halted
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

    fn halt(&mut self) {
        self.halted = true;
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

    fn add(&mut self, a: u16, b: u16, c: u16) {
        self.regfile[self.get_reg(a)] = (self.get_val(b) + self.get_val(c)) % 0x8000;
    }

    fn modulo(&mut self, a: u16, b: u16, c: u16) {
        self.regfile[self.get_reg(a)] = self.get_val(b) % self.get_val(c);
    }

    fn mult(&mut self, a: u16, b: u16, c: u16) {
        self.regfile[self.get_reg(a)] =
            ((self.get_val(b) as u32 * self.get_val(c) as u32) % 0x8000) as u16;
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

    fn rmem(&mut self, a: u16, b: u16) {
        self.regfile[self.get_reg(a)] = self.memory[self.get_val(b) as usize];
    }

    fn wmem(&mut self, a: u16, b: u16) {
        self.memory[self.get_val(a) as usize] = self.get_val(b);
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
        io::stdout().flush().unwrap();
    }

    fn inp(&mut self, a: u16) {
        let mut buf = [0u8; 1];
        io::stdin().read_exact(&mut buf).unwrap();
        self.regfile[self.get_reg(a)] = buf[0] as u16;
    }

    fn noop(&self) {}
}
