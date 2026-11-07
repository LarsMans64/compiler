#![allow(warnings)]

type Register = usize;
type Address = usize;
type Value = u32;

pub enum JumpTarget {
    Instruction(usize),
    Label(String),
    Relative(usize),
}

pub enum Instruction {
    Operation(Operator, Register, Register, Register),
    Load(Address, Register),
    Store(Register, Address),
    LoadImmediate(Value, Register),
    Push(Register),
    Pop(Register),
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
    memory: [u8; 2^20],
    stack_pointer: Address,
    next_output: char,
}

impl ComputerState {
    fn new() -> ComputerState {
        ComputerState {
            pc: 0,
            registers: [0; 16],
            memory: [0; 2^20],
            stack_pointer: 0,
            next_output: 0 as char,
        }
    }

    fn get_mem(&mut self, address: &Address) -> Value {
        // *self.memory.get(*address).unwrap_or(&0);
        todo!()
    }

    fn set_mem(&mut self, address: Address, value: Value) {
        // self.memory.insert(address, value);
        todo!()
    }

    fn push(&mut self, value: Value) {
        // self.memory.insert(self.stack_pointer, value);
        // self.stack_pointer -= 1;
        todo!()
    }

    fn pop(&mut self) -> Value {
        // let it = *self.memory.get(&self.stack_pointer).unwrap_or(&0);
        // self.stack_pointer += 1;
        todo!()
    }
    
    fn jump(&mut self, target: &JumpTarget) {
        match target {
            JumpTarget::Instruction(index) => {
                self.pc = *index;
            }
            JumpTarget::Label(label) => {
                todo!()
            }
            JumpTarget::Relative(offset) => {
                self.pc += *offset;
            }
        }
    }
}

pub fn execute<F>(program: &Vec<Instruction>, output: F)
    where F: Fn(char)
{
    let mut state = ComputerState::new();

    while state.pc < program.len() {
        let instruction = program.get(state.pc).unwrap();
        execute_instruction(&mut state, instruction);
        state.pc += 1;

        if state.next_output as u8 != 0 {
            output(state.next_output);
            state.next_output = 0 as char;
        }
    }
}

fn execute_instruction(state: &mut ComputerState, instruction: &Instruction) {
    match instruction {
        Instruction::Operation(op, a, b, to) => {
            state.registers[*to] = op.compute(state.registers[*a], state.registers[*b]);
        }
        Instruction::Load(a, r) => {
            // state.registers[*r] = *state.memory[a];
        }
        Instruction::Store(r, a) => {
            // state.memory[*a] = state.registers[*r];
        }
        Instruction::LoadImmediate(a, r) => {
            state.registers[*r] = *a;
        }
        Instruction::Push(r) => {
            state.push(state.registers[*r]);
        }
        Instruction::Pop(r) => {
            state.registers[*r] = state.pop();
        }
        Instruction::Jump(t) => {
            state.jump(t);
        }
        Instruction::BranchIfPositive(r, t) => {
            if state.registers[*r] != 0 {
                state.jump(t);
            }
        }
        Instruction::BranchIfZero(r, t) => {
            if state.registers[*r] == 0 {
                state.jump(t);
            }
        }
        Instruction::Malloc(n, to) => {
            todo!()
        }
        Instruction::Free(a, n) => {
            todo!()
        }
        Instruction::Out(r) => state.next_output = state.registers[*r] as u8 as char,
    };
}