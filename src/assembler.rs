use std::collections::HashMap;

type Register = usize;
type Address = usize;
type Offset = usize;
type Value = u32;

pub enum Instruction {
    Operation(Operator, Register, Register, Register),
    Load(Address, Register),
    Store(Register, Address),
    LoadImmediate(Value, Register),
    Push(Register),
    Pop(Register),
    Jump(Offset),
    Branch(Register, Offset),
    BranchIfZero(Register, Offset),
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
    pc: Offset,
    max_address: Address,
    registers: [Value; 16],
    memory: HashMap<Address, Value>,
    stack_pointer: Address,
    next_output: char,
}

impl ComputerState {
    fn new() -> ComputerState {
        ComputerState {
            pc: 0,
            max_address: 2usize.pow(20),
            registers: [0; 16],
            memory: HashMap::new(),
            stack_pointer: 0,
            next_output: 0 as char,
        }
    }

    fn get_mem(&mut self, address: &Address) -> Value {
        *self.memory.get(address).unwrap_or(&0)
    }

    fn set_mem(&mut self, address: Address, value: Value) {
        self.memory.insert(address, value);
    }

    fn push(&mut self, value: Value) {
        self.memory.insert(self.stack_pointer, value);
        self.stack_pointer -= 1;
    }

    fn pop(&mut self) -> Value {
        let it = *self.memory.get(&self.stack_pointer).unwrap_or(&0);
        self.stack_pointer += 1;
        it
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
            state.registers[*r] = *state.memory.get(a).unwrap_or(&0);
        }
        Instruction::Store(r, a) => {
            state.memory.insert(*a, state.registers[*r]);
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
        Instruction::Jump(o) => {
            state.pc += *o;
        }
        Instruction::Branch(r, o) => {
            if state.registers[*r] != 0 {  }
        }
        Instruction::BranchIfZero(r, o) => {}
        Instruction::Malloc(n, to) => {}
        Instruction::Free(a, n) => {}
        Instruction::Out(r) => state.next_output = state.registers[*r] as u8 as char,
    };
}