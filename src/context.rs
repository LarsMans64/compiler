use crate::ast::BinaryOp;
use std::collections::HashMap;

pub type DataSize = u32;

pub type ProgramResult = HashMap<String, (FunctionContext, Body)>;

#[derive(Debug)]
pub struct FunctionContext {
    pub args_size: DataSize,
    pub vars_size: DataSize,
    pub return_size: DataSize,
}

pub type Body = Vec<Action>;
pub type LocalOffset = DataSize;

#[derive(Debug)]
pub enum Action {
    StoreVar(LocalOffset, Vec<SizedComp>, SizedComp),
    StoreArg(LocalOffset, Vec<SizedComp>, SizedComp),
    Call(String, Vec<SizedComp>, DataSize),
    Return(Option<SizedComp>),
    If(SizedComp, Body, Body),
    While(SizedComp, Body),
}

pub type SizedComp = (Box<Computation>, DataSize);

#[derive(Debug)]
pub enum Computation {
    Zero,
    Constant(u32),
    LocalVar(LocalOffset),
    LocalArg(LocalOffset),
    Dereference(SizedComp),
    AddressOfVar(LocalOffset),
    AddressOfArg(LocalOffset),
    Array(Vec<SizedComp>),
    Access(SizedComp, SizedComp),
    Call(String, Vec<SizedComp>),
    Calculation(BinaryOp, SizedComp, SizedComp),
}
