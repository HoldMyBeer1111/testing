#![allow(dead_code)]

use std::collections::HashMap;

use thiserror::Error;

type VariableName = String;
type LabelName = String;

type Instructions = Vec<Instruction>;
type Labels = HashMap<LabelName, usize>;

#[derive(Debug, Clone)]
struct Bytecode {
    pub instrs: Instructions,
    pub labels: Labels,
}

type ValueType = i64;

#[derive(Debug, Clone)]
enum Instruction {
    LoadVal(ValueType),
    WriteVar(VariableName),
    ReadVar(VariableName),
    Add,
    Multiply,
    Subtract,
    Divide,
    ReturnValue,
    JumpIfNeg(LabelName),
    JumpIfPos(LabelName),
    JumpIfZero(LabelName),
    JumpIfNotZero(LabelName),
}

type IpType = usize;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum InterpretationError {
    #[error("operations limit exceeded")]
    OperationsLimitExceeded,

    #[error("stack is empty (IP={0})")]
    StackIsEmpty(IpType),

    #[error("return instruction doesnt exist")]
    ReturnDoesntExist,

    #[error("unknown variable '{var_name:?}' (IP={ip:?})")]
    UnknownVariable { var_name: VariableName, ip: IpType },

    #[error("unknown label '{lbl_name:?}' (IP={ip:?})")]
    UnknownLabel { lbl_name: LabelName, ip: IpType },

    #[error("division by zero (IP={ip:?})")]
    DivisionByZero { ip: IpType },

    #[error("'{val1:?}{op:?}{val2:?}' overflowed (IP={ip:?})")]
    Overflow {
        op: char,
        val1: ValueType,
        val2: ValueType,
        ip: IpType,
    },
}

fn run(bytecode: Bytecode) -> Result<ValueType, InterpretationError> {
    const MAX_OPS: u64 = 1_000;

    let mut stack = vec![];
    let mut vars = HashMap::new();
    let mut ip = 0;
    let mut executed = 0;

    loop {
        executed += 1;
        if executed > MAX_OPS {
            return Err(InterpretationError::OperationsLimitExceeded);
        }

        let instr = bytecode
            .instrs
            .get(ip)
            .cloned()
            .ok_or(InterpretationError::ReturnDoesntExist)?;

        let mut pop_stack = || stack.pop().ok_or(InterpretationError::StackIsEmpty(ip));

        match instr {
            Instruction::LoadVal(val) => stack.push(val),

            Instruction::WriteVar(var_name) => {
                vars.insert(var_name, pop_stack()?);
            }

            Instruction::ReadVar(var_name) => {
                stack.push(
                    vars.get(&var_name)
                        .cloned()
                        .ok_or(InterpretationError::UnknownVariable { var_name, ip })?,
                );
            }

            Instruction::Add => {
                let val1 = pop_stack()?;
                let val2 = pop_stack()?;
                stack.push(
                    val1.checked_add(val2)
                        .ok_or(InterpretationError::Overflow {
                            op: '+',
                            val1,
                            val2,
                            ip,
                        })?,
                );
            }

            Instruction::Subtract => {
                let val1 = pop_stack()?;
                let val2 = pop_stack()?;
                stack.push(
                    val1.checked_sub(val2)
                        .ok_or(InterpretationError::Overflow {
                            op: '-',
                            val1,
                            val2,
                            ip,
                        })?,
                );
            }

            Instruction::Multiply => {
                let val1 = pop_stack()?;
                let val2 = pop_stack()?;
                stack.push(
                    val1.checked_mul(val2)
                        .ok_or(InterpretationError::Overflow {
                            op: '*',
                            val1,
                            val2,
                            ip,
                        })?,
                );
            }

            Instruction::Divide => {
                let val1 = pop_stack()?;
                let val2 = pop_stack()?;
                if val2 == 0 {
                    return Err(InterpretationError::DivisionByZero { ip });
                }
                stack.push(
                    val1.checked_div(val2)
                        .ok_or(InterpretationError::Overflow {
                            op: '/',
                            val1,
                            val2,
                            ip,
                        })?,
                );
            }

            Instruction::JumpIfZero(label) => {
                let val = pop_stack()?;
                if val == 0 {
                    ip = bytecode.labels.get(&label).cloned().ok_or(
                        InterpretationError::UnknownLabel {
                            lbl_name: label,
                            ip,
                        },
                    )?;
                    continue;
                }
            }

            Instruction::JumpIfNotZero(label) => {
                let val = pop_stack()?;
                if val != 0 {
                    ip = bytecode.labels.get(&label).cloned().ok_or(
                        InterpretationError::UnknownLabel {
                            lbl_name: label,
                            ip,
                        },
                    )?;
                    continue;
                }
            }

            Instruction::JumpIfNeg(label) => {
                let val = pop_stack()?;
                if val < 0 {
                    ip = bytecode.labels.get(&label).cloned().ok_or(
                        InterpretationError::UnknownLabel {
                            lbl_name: label,
                            ip,
                        },
                    )?;
                    continue;
                }
            }

            Instruction::JumpIfPos(label) => {
                let val = pop_stack()?;
                if val > 0 {
                    ip = bytecode.labels.get(&label).cloned().ok_or(
                        InterpretationError::UnknownLabel {
                            lbl_name: label,
                            ip,
                        },
                    )?;
                    continue;
                }
            }

            Instruction::ReturnValue => {
                return pop_stack();
            }
        };

        ip += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::task_1_and_2::{run, Bytecode, Instruction, InterpretationError, Labels};

    #[test]
    fn run_fails_when_empty_bytecode() {
        let b = Bytecode {
            instrs: vec![],
            labels: Labels::new(),
        };
        let r = run(b);
        assert_eq!(r, Err(InterpretationError::ReturnDoesntExist));
    }

    #[test]
    fn run_fails_if_unknown_var() {
        let b = Bytecode {
            instrs: vec![Instruction::ReadVar("x".to_owned())],
            labels: Labels::new(),
        };
        let r = run(b);
        assert_eq!(
            r,
            Err(InterpretationError::UnknownVariable {
                var_name: "x".to_owned(),
                ip: 0
            })
        );
    }

    #[test]
    fn run_fails_if_overflow() {
        let b = Bytecode {
            instrs: vec![
                Instruction::LoadVal(i64::MAX),
                Instruction::LoadVal(i64::MAX),
                Instruction::Add,
            ],
            labels: Labels::new(),
        };
        let r = run(b);
        assert_eq!(
            r,
            Err(InterpretationError::Overflow {
                op: '+',
                val1: i64::MAX,
                val2: i64::MAX,
                ip: 2
            }),
        );
    }

    #[test]
    fn run_fails_if_div_by_zero() {
        let b = Bytecode {
            instrs: vec![
                Instruction::LoadVal(0),
                Instruction::LoadVal(i64::MAX),
                Instruction::Divide,
            ],
            labels: Labels::new(),
        };
        let r = run(b);
        assert_eq!(r, Err(InterpretationError::DivisionByZero { ip: 2 }),);
    }

    #[test]
    fn run_fails_if_unknown_label() {
        let b = Bytecode {
            instrs: vec![
                Instruction::LoadVal(0),
                Instruction::JumpIfZero("x".to_owned()),
            ],
            labels: Labels::new(),
        };
        let r = run(b);
        assert_eq!(
            r,
            Err(InterpretationError::UnknownLabel {
                lbl_name: "x".to_owned(),
                ip: 1
            })
        );
    }

    #[test]
    fn run_fails_if_infinite_loop() {
        let mut labels = Labels::new();
        labels.insert("x".to_owned(), 0);
        let b = Bytecode {
            instrs: vec![
                Instruction::LoadVal(0),
                Instruction::JumpIfZero("x".to_owned()),
            ],
            labels,
        };
        let r = run(b);
        assert_eq!(r, Err(InterpretationError::OperationsLimitExceeded {}));
    }

    #[test]
    fn run_fails_if_empty_stack() {
        let b = Bytecode {
            instrs: vec![Instruction::LoadVal(0), Instruction::Add],
            labels: Labels::new(),
        };
        let r = run(b);
        assert_eq!(r, Err(InterpretationError::StackIsEmpty(1)));
    }

    #[test]
    fn run_happy_path() {
        let a_lbl = "a".to_owned();
        let mut labels = Labels::new();
        labels.insert(a_lbl.clone(), 6);
        let (x_var, y_var, z_var) = ("x".to_owned(), "y".to_owned(), "z".to_owned());
        let b = Bytecode {
            instrs: vec![
                Instruction::LoadVal(1),
                Instruction::WriteVar(x_var.clone()),
                Instruction::LoadVal(2),
                Instruction::WriteVar(y_var.clone()),
                Instruction::LoadVal(3),
                Instruction::WriteVar(z_var.clone()),
                Instruction::ReadVar(x_var.clone()),
                Instruction::LoadVal(1),
                Instruction::Add,
                Instruction::WriteVar(x_var.clone()),
                Instruction::LoadVal(1),
                Instruction::ReadVar(z_var.clone()),
                Instruction::Subtract,
                Instruction::WriteVar(z_var.clone()),
                Instruction::ReadVar(z_var),
                Instruction::JumpIfNotZero(a_lbl),
                Instruction::ReadVar(x_var),
                Instruction::ReadVar(y_var),
                Instruction::Multiply,
                Instruction::ReturnValue,
            ],
            labels,
        };
        let r = run(b);
        assert_eq!(r, Ok(8));
    }
}
