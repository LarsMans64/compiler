use crate::context::{Computation, DataSize, FunctionContext, SizedComp};
use crate::ir::{Instr, JumpTarget, Operator, Register};
use std::task::Context;

const REG_ARP: Register = 0;
const REG_A: Register = 1;
const REG_B: Register = 2;

fn generate_function(name: String, context: Context) -> Vec<Instr> {
    let mut code = vec![
        Instr::Pop(REG_A),
        Instr::StoreDynamic(REG_A, REG_ARP, 0),
    ];



    code.push(Instr::Jump(JumpTarget::InstructionDynamic(REG_ARP)));
    code
}

fn generate_call(current_args_size: DataSize, function: &String, arguments: &Vec<SizedComp>) -> Vec<Instr> {
    let mut code = vec![
        Instr::OperationImmediate(Operator::Add, REG_ARP, current_args_size + 1, REG_A)
    ];

    let offset = 0;

    for argument in arguments {
        // code.append(&mut generate_calculation(argument, ...));
        for i in 0..argument.1 {
            code.append(&mut vec![
                Instr::Pop(REG_B),
                Instr::StoreDynamic(REG_B, REG_A, offset + i),
            ]);
        }
    }

    let label = format!("func@{}", function);
    code.append(&mut vec![
        Instr::Copy(REG_A, REG_ARP),
        Instr::OperationImmediate(Operator::Add, REG_ARP, offset, REG_ARP),
        Instr::Call(JumpTarget::Label(label)),
    ]);
    code
}

fn generate_calculation((calc, size): &SizedComp, fun_context: &FunctionContext) -> Vec<Instr> {
    let mut code = Vec::new();
    match &**calc {
        Computation::Constant(val) => {
            code.push(Instr::LoadImmediate(val.clone(), REG_A));
            code.push(Instr::Push(REG_A));
        }
        Computation::LocalVar(index) => {
            for i in 0..*size {
                // code.push(Instr::LoadDynamic(REG_ARP, fun_context.vars_size[index] + i, REG_A));
                code.push(Instr::Push(REG_A));
            }
        }
        Computation::LocalArg(index) => {
            for i in 0..*size {
                // code.push(Instr::LoadDynamic(REG_ARP, fun_context.args_size[index] + i, REG_A));
                code.push(Instr::Push(REG_A));
            }
        }
        Computation::Call(name, args) => {}
        Computation::Dereference(_) => {}
        Computation::AddressOfArg(_) => {}
        Computation::Access(_, ..) => {}
        Computation::Zero => {}
        Computation::AddressOfVar(_) => {}
        Computation::Array(_) => {}
        Computation::Calculation(_, _, _) => {}
    }
    code
}

/*
stack frame

local variables
return address <- activation record pointer
arguments
*/