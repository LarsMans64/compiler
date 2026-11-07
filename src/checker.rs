use crate::ast::{AssignmentTarget, BinaryOp, Call, Expression, Function, Program, Statement, Type, UnaryOperator};
use std::cmp::PartialEq;
use std::collections::HashMap;

#[derive(PartialEq)]
struct FunctionSignature {
    name: String,
    args: Vec<Type>,
    return_type: Option<Type>,
}

type Scope = HashMap<String, VarDef>;

#[derive(Debug, PartialEq)]
struct VarDef {
    name: String,
    typ: Type,
}

type CheckerError = String;

pub struct Checker {
    functions: HashMap<String, FunctionSignature>,
    stack: Vec<Scope>,
}

impl Checker {
    fn push(&mut self) {
        self.stack.push(Scope::new());
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    fn define(&mut self, var: VarDef) {
        let scope = self.stack.last_mut().unwrap();
        if scope.insert(var.name.clone(), var) != None {
            panic!("Defined a variable without checking its existence")
        }
    }

    fn query(&self, name: &str) -> Option<&VarDef> {
        self.stack.iter().rev().find(|scope| scope.get(name).is_some())?.get(name)
    }

    pub fn check_program(program: &Program) -> Result<(), CheckerError> {
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
        };

        for function in program {
            checker.check_function(&function)?;
        }

        Ok(())
    }

    fn check_function(&mut self, fun: &Function) -> Result<(), CheckerError> {
        self.push();

        for (name, typ) in &fun.args {
            self.define(VarDef {
                name: name.clone(),
                typ: typ.clone(),
            });
        }

        for statement in &fun.body {
            self.check_statement(statement, &fun)?;
        }

        self.pop();
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement, function: &Function) -> Result<(), CheckerError> {
        match stmt {
            Statement::Declaration { variable, type_hint, expression } => {
                if let Some(_) = self.query(variable) {
                    return Err(format!("Variable \"{}\" is already declared", variable))
                }

                let found_type = self.check_expression(expression)?;

                if let Some(expected_type) = type_hint {
                    if &found_type != expected_type {
                        return Err(format!("Conflicting types: expected \"{}\" but found \"{}\"", expected_type.show(), found_type.show()));
                    }
                }

                self.define(VarDef {
                    name: variable.clone(),
                    typ: found_type
                });
            }
            Statement::Assignment { target, expression } => {
                let target_type = self.check_assignment_target(target)?;
                let found_type = self.check_expression(expression)?;
                if target_type != found_type {
                    return Err(format!("Cannot assign variable of type \"{}\" to \"{}\"", target_type.show(), found_type.show()));
                }
            }
            Statement::Call(call) => {
                self.check_call(call)?;
            }
            Statement::Return(expression) => {
                let found_type = match expression {
                    Some(e) => Some(self.check_expression(e)?),
                    None => None
                };
                if function.return_type != found_type {
                    let expected = function.return_type.clone().map_or("void".to_string(), |t| t.show());
                    let found = found_type.map_or("void".to_string(), |t| t.show());
                    return Err(format!("Invalid return type: function requires \"{}\" but found \"{}\"", expected, found));
                }
            }
            Statement::If { condition, then, otherwise } => {
                let condition_type = self.check_expression(condition)?;
                if condition_type != Type::Boolean {
                    return Err(format!("Expected condition, found \"{}\"", condition_type.show()));
                }

                self.push();
                for statement in then {
                    self.check_statement(statement, function)?;
                }
                self.pop();

                self.push();
                for statement in otherwise {
                    self.check_statement(statement, function)?;
                }
                self.pop();
            }
            Statement::While { condition, then } => {
                let condition_type = self.check_expression(condition)?;
                if condition_type != Type::Boolean {
                    return Err(format!("Expected condition, found \"{}\"", condition_type.show()));
                }

                self.push();
                for statement in then {
                    self.check_statement(statement, function)?;
                }
                self.pop();
            }
            Statement::Body(statements) => {
                self.push();
                for statement in statements {
                    self.check_statement(statement, function)?;
                }
                self.pop();
            }
        }
        Ok(())
    }
    
    fn check_assignment_target(&mut self, target: &AssignmentTarget) -> Result<Type, CheckerError> {
        match target {
            AssignmentTarget::Index(target, index) => {
                let target_type = self.check_assignment_target(target)?;
                let index_type = self.check_expression(index)?;
                if index_type != Type::Integer {
                    return Err(format!("Type \"{}\" cannot be indexed by \"{}\"", target_type.show(), index_type.show()))
                }
                
                if let Type::Array(inner) = target_type {
                    return Ok(*inner)
                }
                Err(format!("Type \"{}\" cannot be indexed", target_type.show()))
            }
            AssignmentTarget::Dereference(target) => {
                let target_type = self.check_assignment_target(target)?;
                if let Type::Pointer(inner) = target_type {
                    return Ok(*inner)
                }
                Err(format!("Type \"{}\" cannot be indexed", target_type.show()))
            }
            AssignmentTarget::Variable(name) => {
                let Some(var) = self.query(name) else {
                    return Err(format!("Variable \"{}\" does not exist", name))
                };
                Ok(var.typ.clone())
            }
        }
    }

    fn check_expression(&self, expr: &Expression) -> Result<Type, CheckerError> {
        match expr {
            Expression::BinaryOperation(e1, op, e2) => {
                let type1 = self.check_expression(e1)?;
                let type2 = self.check_expression(e2)?;
                match op {
                    BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Times | BinaryOp::DividedBy | BinaryOp::RaisedTo => {
                        if type1 == Type::Integer && type2 == Type::Integer {
                            return Ok(Type::Integer);
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if type1 == Type::Integer && type2 == Type::Integer {
                            return Ok(Type::Integer);
                        }
                        if type1 == Type::Boolean && type2 == Type::Boolean {
                            return Ok(Type::Boolean);
                        }
                    }
                    BinaryOp::Eq | BinaryOp::Neq => {
                        return Ok(Type::Boolean);
                    }
                    BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Lte | BinaryOp::Gte => {
                        if type1 == Type::Integer && type2 == Type::Integer {
                            return Ok(Type::Boolean);
                        }
                    }
                };
                Err(format!("Cannot perform {} to \"{}\" and \"{}\"", op.show(), type1.show(), type2.show()))
            }
            Expression::UnaryOperation(op, expr) => {
                let typ = self.check_expression(expr)?;
                match op {
                    UnaryOperator::Minus => {
                        if typ == Type::Integer {
                            return Ok(Type::Integer);
                        }
                    }
                    UnaryOperator::Not => {
                        if typ == Type::Boolean {
                            return Ok(Type::Boolean);
                        }
                    }
                    UnaryOperator::AddressOf => {
                        return Ok(Type::Pointer(Box::new(typ)));
                    }
                    UnaryOperator::Dereference => {
                        if let Type::Pointer(inner) = typ {
                            return Ok(*inner);
                        }
                    }
                }
                Err(format!("Cannot perform {} to \"{}\"", op.show(), typ.show()))
            }
            Expression::Array(values) => {
                let first_type = self.check_expression(values.first().unwrap())?;
                for value in values.iter().skip(1) {
                    let other_type = self.check_expression(value)?;
                    if other_type != first_type {
                        return Err(format!(
                            "Array can't have more than one type: first type is \"{}\" but also found \"{}\"",
                            first_type.show(),
                            other_type.show(),
                        ));
                    }
                }
                Ok(Type::Array(Box::new(first_type)))
            }
            Expression::Access(expr, index) => {
                let expr_type = self.check_expression(expr)?;
                let index_type = self.check_expression(index)?;

                let Type::Integer = &index_type else {
                    return Err(format!("Type \"{}\" cannot be indexed by \"{}\"", expr_type.show(), index_type.show()));
                };

                let Type::Array(inner) = expr_type else {
                    return Err(format!("Type \"{}\" cannot be indexed", expr_type.show()));
                };

                Ok(*inner)
            }
            Expression::Call(call) => {
                if let Some(typ) = self.check_call(call)? {
                    return Ok(typ)
                }
                Err(format!("Function \"{}\" has no return type", call.0))
            }
            Expression::Integer(_) => Ok(Type::Integer),
            Expression::Boolean(_) => Ok(Type::Boolean),
            Expression::Character(_) => Ok(Type::Character),
            Expression::String(_) => Ok(Type::Array(Box::new(Type::Character))),
            Expression::Variable(name) => {
                if let Some(var) = self.query(name) {
                    return Ok(var.typ.clone());
                };
                Err(format!("Variable \"{}\" does not exist", name))
            }
        }
    }
    
    fn check_call(&self, (name, args): &Call) -> Result<Option<Type>, CheckerError> {
        let Some(signature) = self.functions.get(name) else {
            return Err(format!("Function \"{}\" does not exist", name))
        };

        if signature.args.len() != args.len() {
            return Err(format!("Function \"{}\" requires {} arguments but found {}", name, signature.args.len(), args.len()));
        }

        for i in 0..args.len() {
            let expected_type = &signature.args[i];
            let found_type = self.check_expression(&args[i])?;
            if expected_type != &found_type {
                return Err(format!("Invalid argument type: expected \"{}\" but found \"{}\"", expected_type.show(), found_type.show()));
            }
        }

        Ok(signature.return_type.clone())
    }
}

fn builtins() -> HashMap<String, FunctionSignature> {
    let builtins = [
        FunctionSignature {name: "print".to_string(), args: vec![Type::Pointer(Box::new(Type::Character))], return_type: None},
        FunctionSignature {name: "println".to_string(), args: vec![Type::Pointer(Box::new(Type::Character))], return_type: None},
        FunctionSignature {name: "malloc".to_string(), args: vec![Type::Integer], return_type: Some(Type::Pointer(Box::new(Type::Integer)))},
    ];
    builtins.into_iter().map(|s| (s.name.clone(), s)).collect()
}
