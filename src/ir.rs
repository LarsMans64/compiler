#![allow(warnings)]

pub type Register = u32;
pub type Address = u32;
pub type Offset = u32;
pub type Value = u32;

pub enum JumpTarget {
    Instruction(usize),
    Relative(usize),
    InstructionDynamic(Register),
    RelativeDynamic(Register),
    Label(String),
}

pub enum Instr {
    Operation(Operator, Register, Register, Register),
    OperationImmediate(Operator, Register, Value, Register),
    Copy(Register, Register),
    LoadImmediate(Value, Register),
    LoadStatic(Address, Register),
    StoreStatic(Register, Address),
    LoadDynamic(Register, Offset, Register),
    StoreDynamic(Register, Register, Offset),
    Push(Register),
    Pop(Register),
    Call(JumpTarget),
    Jump(JumpTarget),
    BranchIfPositive(Register, JumpTarget),
    BranchIfZero(Register, JumpTarget),
    Malloc(Register, Register),
    Free(Register, Register),
    Out(Register),
}

pub enum Operator {
    Add, Sub, Mul, Div, Mod, And, Or, Xor,
}

impl Operator {
    fn compute(&self, a: Value, b: Value) -> Value {
        match self {
            Operator::Add => a + b,
            Operator::Sub => a - b,
            Operator::Mul => a * b,
            Operator::Div => a / b,
            Operator::Mod => a % b,
            Operator::And => a & b,
            Operator::Or => a | b,
            Operator::Xor => a ^ b,
        }
    }
}

struct ComputerState {
    pc: usize,
    registers: [Value; 16],
    memory: [Value; 2^20],
    stack_pointer: Address,
    next_output: Option<char>,
}

impl ComputerState {
    fn new() -> ComputerState {
        ComputerState {
            pc: 0,
            registers: [0; 16],
            memory: [0; 2^20],
            stack_pointer: 0,
            next_output: None,
        }
    }

    fn get_register(&self, register: &Register) -> Value {
        // self.registers[*register as usize]
        todo!()
    }
    
    fn set_register(&mut self, register: &Register, value: Value) {
        // self.registers[*register as usize] = value;
        todo!()
    }

    fn get_mem(&self, address: &Address) -> Value {
        self.memory[*address as usize]
    }

    fn set_mem(&mut self, address: &Address, value: Value) {
        self.memory[*address as usize] = value;
    }

    fn push(&mut self, value: Value) {
        self.memory[self.stack_pointer as usize] = value;
        self.stack_pointer -= 1;
    }

    fn pop(&mut self) -> Value {
        let it = self.memory[self.stack_pointer as usize];
        self.stack_pointer += 1;
        it
    }

    fn jump(&mut self, target: &JumpTarget) {
        match target {
            JumpTarget::Instruction(index) => {
                self.pc = *index - 1;
            }
            JumpTarget::Relative(offset) => {
                self.pc += *offset - 1;
            }
            JumpTarget::InstructionDynamic(register) => {
                self.pc = self.get_register(register) as usize - 1;
            }
            JumpTarget::RelativeDynamic(register) => {
                self.pc += self.get_register(register) as usize - 1;
            }
            JumpTarget::Label(label) => {
                todo!()
            }
        }
    }
}

pub fn execute<F>(program: &Vec<Instr>, output: F)
    where F: Fn(char)
{
    let mut state = ComputerState::new();

    while state.pc < program.len() {
        let instruction = program.get(state.pc).unwrap();
        execute_instruction(&mut state, instruction);
        state.pc += 1;

        if let Some(c) = state.next_output {
            output(c);
            state.next_output = None;
        }
    }
}

fn execute_instruction(state: &mut ComputerState, instruction: &Instr) {
    match instruction {
        Instr::Operation(op, a, b, to) => {
            state.set_register(to, op.compute(state.get_register(a), state.get_register(b)));
        }
        Instr::OperationImmediate(op, a, imm, to) => {
            state.set_register(to, op.compute(state.get_register(a), *imm));
        }
        Instr::Copy(a, b) => {
            state.set_register(b, state.get_register(a));
        }
        Instr::LoadImmediate(a, r) => {
            state.set_register(r,  *a);
        }
        Instr::LoadStatic(a, r) => {
            state.set_register(r, state.get_mem(a));
        }
        Instr::StoreStatic(r, a) => {
            state.set_mem(a, state.get_register(r));
        }
        Instr::LoadDynamic(from, offset, to) => {
            let address = state.get_register(from) + offset;
            state.set_register(to, state.get_mem(&address));
        }
        Instr::StoreDynamic(from, to, offset) => {
            let address = state.get_register(to) + offset;
            state.set_mem(&address, state.get_register(from));
        }
        Instr::Push(r) => {
            state.push(state.get_register(r));
        }
        Instr::Pop(r) => {
            let value = state.pop();
            state.set_register(r, value);
        }
        Instr::Call(t) => {
            state.push(state.pc as Value + 1);
            state.jump(t);
        }
        Instr::Jump(t) => {
            state.jump(t);
        }
        Instr::BranchIfPositive(r, t) => {
            if state.get_register(r) != 0 {
                state.jump(t);
            }
        }
        Instr::BranchIfZero(r, t) => {
            if state.get_register(r) == 0 {
                state.jump(t);
            }
        }
        Instr::Malloc(n, to) => {
            todo!()
        }
        Instr::Free(a, n) => {
            todo!()
        }
        Instr::Out(r) => { 
            state.next_output = Some(state.get_register(r) as u8 as char);
        }
    };
}