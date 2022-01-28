use std::{collections::HashMap, io::Write, rc::Rc};

use crate::{
    chunk::{IdentifierName, Operation},
    native::clock,
    object::{ObjFunction, ObjNative, ObjString},
    stack::Stack,
    value::Value,
};

#[derive(Debug)]
pub enum RuntimeError {
    NoMoreOperations(usize),
    Other(String),
}

impl RuntimeError {
    pub fn new(message: &str) -> Self {
        Self::Other(message.to_string())
    }
}

pub type InterpretResult<V> = Result<V, RuntimeError>;

#[derive(Clone)]
struct CallFrame<'a> {
    function: &'a ObjFunction,
    ip: usize,
    first_slot: usize,
}

impl<'a> CallFrame<'a> {
    pub fn new(function: &'a ObjFunction, first_slot: usize) -> Self {
        CallFrame {
            function,
            ip: 0,
            first_slot,
        }
    }
}

pub struct VM {
    stack: Stack,
    globals: HashMap<IdentifierName, Value>,
    call_stack: Vec<String>,
}

impl VM {
    pub fn new() -> Self {
        let mut ret = VM {
            stack: Stack::new(),
            globals: HashMap::new(),
            call_stack: vec![],
        };
        ret.define_native("clock", clock).unwrap();
        ret
    }

    pub fn run<W: Write>(&mut self, function: &ObjFunction, output: &mut W) -> InterpretResult<()> {
        let mut frame = CallFrame::new(function, self.call_stack.len());
        self.call_stack.push(function.name.clone());

        let code = frame.function.chunk.code();
        let chunk = &frame.function.chunk;

        loop {
            let op = code.get(frame.ip)
                .ok_or(RuntimeError::NoMoreOperations(frame.ip))?;
            frame.ip += 1;

            #[cfg(feature = "trace")]
            {
                writeln!(output, "          {:?}", self.stack);
                op.disassemble(&chunk, frame.ip - 1);
            }

            match op {
                Operation::Constant(iid) => {
                    let c = chunk.read_constant(*iid);
                    self.stack.push(c.clone());
                }
                Operation::Nil => self.stack.push(Value::Nil),
                Operation::True => self.stack.push(Value::Boolean(true)),
                Operation::False => self.stack.push(Value::Boolean(false)),
                Operation::Pop => {
                    self.stack.pop()?; //.expect("There was nothing to pop");
                }
                Operation::GetGlobal(name) => {
                    let val = self
                        .globals
                        .get(name)
                        .ok_or(RuntimeError::new(&format!("Undefined variable '{}'", name)))?;
                    self.stack.push(val.clone());
                }
                Operation::DefineGlobal(name) => {
                    self.globals.insert(name.clone(), self.stack.pop()?);
                }
                Operation::SetGlobal(name) => {
                    if !self.globals.contains_key(name) {
                        return Err(RuntimeError::new(&format!("Undefined variable '{}'", name)));
                    }

                    self.globals
                        .insert(name.clone(), self.stack.peek()?.clone());
                }
                Operation::GetLocal(i) => {
                    let absolute_index = i + frame.first_slot;
                    let val = self
                        .stack
                        .get(absolute_index)?
                        // .expect("Local variable not found in the stack")
                        .clone();
                    self.stack.push(val);
                }
                Operation::SetLocal(i) => {
                    let absolute_index = frame.first_slot + i;
                    let val = self
                        .stack
                        .peek()?
                        // .expect("Expression not found in the stack to assign to local")
                        .clone();
                    self.stack.set(absolute_index, val);
                }
                Operation::Equal => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(Value::Boolean(a == b));
                }
                Operation::Greater => VM::binary(&mut self.stack, |a, b| Value::Boolean(a > b))?,
                Operation::Less => VM::binary(&mut self.stack, |a, b| Value::Boolean(a < b))?,
                Operation::Add => match self.stack.peek()? {
                    Value::Number(_) => VM::binary(&mut self.stack, |a, b| Value::Number(a + b))?,
                    Value::String(_) => {
                        let b = self.stack.pop_string()?;
                        let a = self.stack.pop_string()?;
                        let result = format!("{}{}", a.value(), b.value());
                        let value = Value::String(Rc::from(ObjString::from_owned(result)));
                        self.stack.push(value);
                    }
                    v => Err(RuntimeError::new(&format!("Can't add the operand {:?}", v)))?,
                },
                Operation::Substract => VM::binary(&mut self.stack, |a, b| Value::Number(a - b))?,
                Operation::Multiply => VM::binary(&mut self.stack, |a, b| Value::Number(a * b))?,
                Operation::Divide => VM::binary(&mut self.stack, |a, b| Value::Number(a / b))?,
                Operation::Not => {
                    let old = self.stack.pop()?;
                    let new = old.is_falsey();
                    self.stack.push(Value::Boolean(new));
                }
                Operation::Negate => {
                    let n = self.stack.pop_number()?;
                    let res = -n;
                    self.stack.push(Value::Number(res));
                }
                Operation::Print => {
                    writeln!(
                        output,
                        "{}",
                        self.stack.pop()? // .expect("Tried to print a non-existing value")
                    )
                    .map_err(|x| {
                        RuntimeError::new(&format!(
                            "Unexpected error while printing to output: {}",
                            x
                        ))
                    })?;
                }
                Operation::Return => {
                    // let result = self.stack.pop()?;
                    self.call_stack.pop();
                    return Ok(());
                }
                Operation::JumpIfFalse(offset) => {
                    let exp = self.stack.peek()?; //.expect("Missing the if expression");
                    if exp.is_falsey() {
                        frame.ip += offset;
                    }
                }
                Operation::Jump(offset) => frame.ip += offset,
                Operation::Loop(offset) => frame.ip -= offset,
                Operation::Call(arg_count) => {
                    let callee = self.stack.peek_many(*arg_count as usize)?.clone();
                    self.call_value(&callee, *arg_count, output)?;
                }
            }
        }
    }

    fn binary<F>(stack: &mut Stack, implementation: F) -> InterpretResult<()>
    where
        F: Fn(f64, f64) -> Value,
    {
        let b = stack.pop_number()?;
        let a = stack.pop_number()?;
        let result = implementation(a, b);
        stack.push(result);
        Ok(())
    }

    fn call_value<W: Write>(
        &mut self,
        callee: &Value,
        arg_count: u8,
        output: &mut W,
    ) -> InterpretResult<()> {
        match callee {
            Value::Function(fun) => {
                self.run(fun, output)?;
                // self.stack.pop()?;
                Ok(())
            }
            Value::Native(native) => {
                let result = (native.function)();
                self.stack.push(Value::Number(result));
                Ok(())
            }
            other => Err(RuntimeError::new(&format!(
                "Expected a function or a class to call, but found {}",
                other
            ))),
        }
    }

    fn define_native(&mut self, name: &str, function: fn() -> f64) -> InterpretResult<()> {
        let obj_native = ObjNative::new(name, function);
        let native = Value::Native(obj_native);
        self.globals.insert(name.to_string(), native);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::VM;
    use crate::{chunk::Operation, object::ObjFunction, value::Value, vm::RuntimeError};
    use std::io;

    #[test]
    fn constants() {
        assert_stack(&mut vec![Operation::True], vec![Value::Boolean(true)]);
    }

    ////////////////

    fn assert_stack(operations: &mut Vec<Operation>, stack: Vec<Value>) {
        let mut function = ObjFunction::new("test");
        function.chunk.emit_many(operations);

        let mut stdout = io::stdout();

        let mut vm = VM::new();
        match vm.run(&function, &mut stdout) {
            Ok(_) => panic!("Expected the VM to halt but it didn't"),
            Err(RuntimeError::NoMoreOperations(_)) => {
                assert_eq!(
                    vm.stack.contents(),
                    &stack,
                    "Stack contents are not the same"
                )
            }
            Err(other) => panic!(
                "Expected the VM to halt but another error happened: {:?}",
                other
            ),
        }
    }
}
