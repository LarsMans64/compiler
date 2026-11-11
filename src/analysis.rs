use crate::ast::{AssignmentTarget, BinaryOp, Call, Expression, Function, Program, Statement, Type, UnaryOperator};
use crate::context::{Action, Body, Computation, DataSize, FunctionContext, LocalOffset, ProgramResult, SizedComp};
use std::cmp::PartialEq;
use std::collections::HashMap;
#[derive(PartialEq)]
struct FunctionSignature {
    name: String,
    args: Vec<Type>,
    return_type: Option<Type>,
}

type Scope = HashMap<String, (VarDef, LocalOffset)>;

#[derive(Debug, PartialEq)]
struct VarDef {
    name: String,
    typ: Type,
    is_arg: bool,
}

type CheckerError = String;

pub struct Checker {
    functions: HashMap<String, FunctionSignature>,
    stack: Vec<Scope>,
    var_offset: LocalOffset,
    arg_offset: LocalOffset,
}

impl Checker {
    fn push_scope(&mut self) {
        self.stack.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.stack.pop();
    }

    fn define_var(&mut self, var: VarDef) -> LocalOffset {
        let scope = self.stack.last_mut().unwrap();
        let offset = if var.is_arg {
            let old = self.arg_offset;
            self.arg_offset += var.typ.size() as LocalOffset;
            old
        } else {
            let old = self.var_offset;
            self.var_offset += var.typ.size() as LocalOffset;
            old
        };
        if scope.insert(var.name.clone(), (var, offset)) != None {
            panic!("Defined a variable without checking its existence")
        }
        offset
    }

    fn query_var(&self, name: &str) -> Result<&(VarDef, LocalOffset), CheckerError> {
        let Some(scope) = self.stack.iter().rev().find(|scope| scope.get(name).is_some()) else {
            return Err(format!("Variable \"{}\" does not exist", name))
        };
        Ok(scope.get(name).unwrap())
    }

    pub fn check_program(program: &Program) -> Result<ProgramResult, CheckerError> {
        let mut funs = builtins();
        for fun in program {
            if funs.contains_key(&fun.name) {
                return Err(format!("Duplicate definition of function {}", fun.name));
            }
            funs.insert(
                fun.name.clone(),
                FunctionSignature {
                    name: fun.name.clone(),
                    args: fun.args.iter().map(|(_, t)| t.clone()).collect(),
                    return_type: fun.return_type.clone(),
                }
            );
        }

        let mut checker = Checker {
            functions: funs,
            stack: Vec::new(),
            var_offset: 0,
            arg_offset: 0,
        };

        let mut result = ProgramResult::new();
        for function in program {
            result.insert(function.name.clone(), checker.check_function(&function)?);
        }

        Ok(result)
    }

    fn check_function(&mut self, fun: &Function) -> Result<(FunctionContext, Body), CheckerError> {
        self.push_scope();

        for (name, typ) in &fun.args {
            self.define_var(VarDef {
                name: name.clone(),
                typ: typ.clone(),
                is_arg: true,
            });
        }

        let mut body = Vec::new();
        for statement in &fun.body {
            body.push(self.check_statement(statement, &fun)?);
        }

        self.pop_scope();

        let function_context = FunctionContext {
            args_size: self.arg_offset,
            vars_size: self.var_offset,
            return_size: fun.return_type.clone().map_or(0, |t| t.size()),
        };

        Ok((function_context, body))
    }

    fn check_statement(&mut self, stmt: &Statement, function: &Function) -> Result<Action, CheckerError> {
        Ok(match stmt {
            Statement::Declaration { variable, type_hint, expression } => {
                if let Ok(_) = self.query_var(variable) {
                    return Err(format!("Variable \"{}\" is already declared", variable))
                }

                let (found_type, comp) = self.check_expression(expression)?;

                if let Some(expected_type) = type_hint {
                    if &found_type != expected_type {
                        return Err(format!("Conflicting types: expected \"{}\" but found \"{}\"", expected_type.show(), found_type.show()));
                    }
                }

                let var = VarDef {
                    name: variable.clone(),
                    typ: found_type,
                    is_arg: false
                };

                let offset = self.define_var(var);

                Action::StoreVar(offset, Vec::new(), comp)
            }
            Statement::Assignment { target, expression } => {
                let (target_type, name, indices) = self.check_assignment_target(target)?;
                let (found_type, comp) = self.check_expression(expression)?;
                if target_type != found_type {
                    return Err(format!("Cannot assign variable of type \"{}\" to \"{}\"", target_type.show(), found_type.show()));
                }
                let (var, offset) = self.query_var(&name)?;
                if var.is_arg {
                    Action::StoreArg(*offset, indices, comp)
                } else {
                    Action::StoreVar(*offset, indices, comp)
                }
            }
            Statement::Call(call) => {
                let (typ, comps) = self.check_call(call)?;
                let return_size = typ.map_or_default(|t| t.size());
                Action::Call(call.0.clone(), comps, return_size)
            }
            Statement::Return(expression) => {
                let found_type = match expression {
                    Some(e) => Some(self.check_expression(e)?),
                    None => None
                };

                if let Some(expected) = &function.return_type {
                    let Some((found, comp)) = found_type else {
                        return Err(format!("Invalid return type: expression expected of type \"{}\"", expected.show()));
                    };
                    if &found != expected {
                        return Err(format!("Invalid return type: function requires \"{}\" but found \"{}\"", expected.show(), found.show()));
                    }
                    Action::Return(Some(comp))
                } else {
                    if let Some((found, _)) = found_type {
                        return Err(format!("Invalid return type: function has no return type, but found \"{}\"", found.show()));
                    };
                    Action::Return(None)
                }
            }
            Statement::If { condition, then, otherwise } => {
                let (condition_type, comp) = self.check_expression(condition)?;
                if condition_type != Type::Boolean {
                    return Err(format!("Expected condition, found \"{}\"", condition_type.show()));
                }

                let mut then_body = Vec::new();
                let mut otherwise_body = Vec::new();

                self.push_scope();
                for statement in then {
                    then_body.push(self.check_statement(statement, function)?);
                }
                self.pop_scope();

                self.push_scope();
                for statement in otherwise {
                    otherwise_body.push(self.check_statement(statement, function)?);
                }
                self.pop_scope();

                Action::If(comp, then_body, otherwise_body)
            }
            Statement::While { condition, then } => {
                let (condition_type, comp) = self.check_expression(condition)?;
                if condition_type != Type::Boolean {
                    return Err(format!("Expected condition, found \"{}\"", condition_type.show()));
                }

                let mut then_body = Vec::new();
                self.push_scope();
                for statement in then {
                    then_body.push(self.check_statement(statement, function)?);
                }
                self.pop_scope();
                Action::While(comp, then_body)
            }
        })
    }

    fn check_assignment_target(&mut self, target: &AssignmentTarget) -> Result<(Type, String, Vec<SizedComp>), CheckerError> {
        match target {
            AssignmentTarget::Index(target, index) => {
                let (target_type, name, mut indices) = self.check_assignment_target(target)?;
                let (index_type, comp) = self.check_expression(index)?;
                if index_type != Type::Integer {
                    return Err(format!("Type \"{}\" cannot be indexed by \"{}\"", target_type.show(), index_type.show()))
                }
                if let Type::Array(inner) = target_type {
                    indices.push(comp);
                    return Ok((*inner, name, indices));
                }
                Err(format!("Type \"{}\" cannot be indexed", target_type.show()))
            }
            AssignmentTarget::Dereference(target) => {
                let (target_type, name, mut indices) = self.check_assignment_target(target)?;
                if let Type::Pointer(inner) = target_type {
                    indices.push(sized(Computation::Constant(0), Type::Integer.size()));
                    return Ok((*inner, name, indices));
                }
                Err(format!("Type \"{}\" cannot be indexed", target_type.show()))
            }
            AssignmentTarget::Variable(name) => {
                let (var, _) = self.query_var(name)?;
                Ok((var.typ.clone(), var.name.clone(), Vec::new()))
            }
        }
    }

    fn check_expression(&self, expr: &Expression) -> Result<(Type, SizedComp), CheckerError> {
        match expr {
            Expression::BinaryOperation(e1, op, e2) => {
                let (type1, comp1) = self.check_expression(e1)?;
                let (type2, comp2) = self.check_expression(e2)?;
                let comp = Computation::Calculation(op.clone(), comp1, comp2);
                match op {
                    BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Times | BinaryOp::DividedBy => {
                        if type1 == Type::Integer && type2 == Type::Integer {
                            return Ok((Type::Integer, sized(comp, Type::Integer.size())));
                        }
                    }
                    BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                        if type1 == Type::Integer && type2 == Type::Integer {
                            return Ok((Type::Integer, sized(comp, Type::Integer.size())));
                        }
                        if type1 == Type::Boolean && type2 == Type::Boolean {
                            return Ok((Type::Boolean, sized(comp, Type::Boolean.size())));
                        }
                    }
                    BinaryOp::Eq | BinaryOp::Neq => {
                        return Ok((Type::Boolean, sized(comp, Type::Boolean.size())));
                    }
                    BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Lte | BinaryOp::Gte => {
                        if type1 == Type::Integer && type2 == Type::Integer {
                            return Ok((Type::Boolean, sized(comp, Type::Boolean.size())));
                        }
                    }
                };
                Err(format!("Cannot perform {} to \"{}\" and \"{}\"", op.show(), type1.show(), type2.show()))
            }
            Expression::UnaryOperation(op, expr) => {
                let (typ, comp) = self.check_expression(expr)?;
                match op {
                    UnaryOperator::Minus => {
                        if typ == Type::Integer {
                            return Ok((Type::Integer, sized(Computation::Calculation(
                                BinaryOp::Minus,
                                sized(Computation::Zero, Type::Integer.size()),
                                comp
                            ), Type::Integer.size())));
                        }
                    }
                    UnaryOperator::Not => {
                        if typ == Type::Boolean {
                            return Ok((Type::Boolean, sized(Computation::Calculation(
                                BinaryOp::Xor,
                                sized(Computation::Constant(1), Type::Boolean.size()),
                                comp
                            ), Type::Boolean.size())));
                        }
                    }
                    UnaryOperator::Dereference => {
                        if let Type::Pointer(inner) = typ {
                            let inner_size = inner.size();
                            return Ok((*inner, sized(Computation::Dereference(comp), inner_size)));
                        }
                    }
                }
                Err(format!("Cannot perform {} to \"{}\"", op.show(), typ.show()))
            }
            Expression::Array(values) => {
                let (first_type, first_comp) = self.check_expression(values.first().unwrap())?;
                let mut contexts = vec![first_comp];
                for value in values.iter().skip(1) {
                    let (other_type, comp) = self.check_expression(value)?;
                    if other_type != first_type {
                        return Err(format!(
                            "Array can't have more than one type: first type is \"{}\" but also found \"{}\"",
                            first_type.show(),
                            other_type.show(),
                        ));
                    }
                    contexts.push(comp)
                }
                let array_type = Type::Array(Box::new(first_type));
                let array_context = sized(Computation::Array(contexts), array_type.size());
                Ok((array_type, array_context))
            }
            Expression::Access(expr, index) => {
                let (expr_type, comp1) = self.check_expression(expr)?;
                let (index_type, comp2) = self.check_expression(index)?;

                let Type::Integer = &index_type else {
                    return Err(format!("Type \"{}\" cannot be indexed by \"{}\"", expr_type.show(), index_type.show()));
                };

                let Type::Array(inner) = expr_type else {
                    return Err(format!("Type \"{}\" cannot be indexed", expr_type.show()));
                };

                let inner_size = inner.size();
                Ok((*inner, sized(Computation::Access(comp1, comp2), inner_size)))
            }
            Expression::Call(call) => {
                if let (Some(typ), comps) = self.check_call(call)? {
                    let size = typ.size();
                    return Ok((typ, sized(Computation::Call(call.0.clone(), comps), size)))
                }
                Err(format!("Function \"{}\" does not return anything", call.0))
            }
            Expression::AddressOf(name) => {
                let (var, offset) = self.query_var(name)?;
                let typ = Type::Pointer(Box::new(var.typ.clone()));
                let comp = if var.is_arg {
                    Computation::AddressOfArg(*offset)
                } else {
                    Computation::AddressOfVar(*offset)
                };
                let size = typ.size();
                Ok((typ, sized(comp, size)))
            }
            Expression::Variable(name) => {
                let (var, offset) = self.query_var(name)?;
                let typ = var.typ.clone();
                let comp = if var.is_arg {
                    Computation::LocalVar(*offset)
                } else {
                    Computation::LocalArg(*offset)
                };
                let size = typ.size();
                Ok((typ, sized(comp, size)))
            }
            Expression::Integer(value) => Ok((Type::Integer, sized(Computation::Constant(*value), Type::Integer.size()))),
            Expression::Boolean(value) => Ok((Type::Boolean, sized(Computation::Constant(if *value { 1 } else { 0 }), Type::Boolean.size()))),
            Expression::Character(value) => Ok((Type::Character, sized(Computation::Constant(*value as u8 as u32), Type::Character.size()))),
            Expression::String(value) => {
                let characters = value.chars().map(|char| {
                    sized(Computation::Constant(char as u32), Type::Character.size())
                }).collect();
                let typ = Type::Array(Box::new(Type::Character));
                let size = typ.size();
                Ok((typ, sized(Computation::Array(characters), size)))
            }
        }
    }

    fn check_call(&self, (name, args): &Call) -> Result<(Option<Type>, Vec<SizedComp>), CheckerError> {
        let Some(signature) = self.functions.get(name) else {
            return Err(format!("Function \"{}\" does not exist", name))
        };

        if signature.args.len() != args.len() {
            return Err(format!("Function \"{}\" requires {} arguments but found {}", name, signature.args.len(), args.len()));
        }

        let mut comps = Vec::new();
        for i in 0..args.len() {
            let expected_type = &signature.args[i];
            let (found_type, comp) = self.check_expression(&args[i])?;
            if expected_type != &found_type {
                return Err(format!("Invalid argument type: expected \"{}\" but found \"{}\"", expected_type.show(), found_type.show()));
            }
            comps.push(comp);
        }

        Ok((signature.return_type.clone(), comps))
    }
}

fn sized(c: Computation, size: DataSize) -> SizedComp {
    (Box::new(c), size)
}

fn builtins() -> HashMap<String, FunctionSignature> {
    let builtins = [
        FunctionSignature {name: "print".to_string(), args: vec![Type::Array(Box::new(Type::Character))], return_type: None},
        FunctionSignature {name: "println".to_string(), args: vec![Type::Array(Box::new(Type::Character))], return_type: None},
        FunctionSignature {name: "heap".to_string(), args: vec![Type::Integer], return_type: Some(Type::Pointer(Box::new(Type::Integer)))},
    ];
    builtins.into_iter().map(|s| (s.name.clone(), s)).collect()
}
