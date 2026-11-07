use crate::assembly::{Instruction, JumpTarget};
use crate::ast::Type;
use std::collections::HashMap;

struct StackFrame {
    variable_offsets: HashMap<String, usize>
}

enum Action {
    StoreVariable(String, Box<Calculation>, usize),
}

enum Calculation {
    Immediate(u32),
    LoadVariable(String, usize),
    CompareEqual(Box<Calculation>, Box<Calculation>, usize),
    CompareNotEqual(Box<Calculation>, Box<Calculation>, usize),
    Call(String, Box<Calculation>, usize),
}

impl Calculation {
    fn get_returned_size(&self) -> usize {
        todo!()
    }
}

static REG_ARP: usize = 1;
static REG_A: usize = 2;
static REG_B: usize = 3;

fn size_of(t: &Type) -> usize {
    match t {
        Type::Integer => 4,
        Type::Boolean => 1,
        Type::Character => 1,
        Type::Pointer(_) => 1,
        Type::Array(_) => 1,
    }
}

fn generate_call(function: String, arguments: &Vec<Calculation>) -> Vec<Instruction> {
    let mut code = Vec::new();

    for argument in arguments {
        code.append(&mut generate_calculation(argument));
        code.push(Instruction::Pop(REG_A));
    }

    code.push(Instruction::Jump(JumpTarget::Label(function)));
    code
}

fn generate_calculation(argument: &Calculation) -> Vec<Instruction> {
    todo!()
}

/*
stack frame

local variables
return address <- activation record pointer
arguments
*/