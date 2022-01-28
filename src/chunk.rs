use std::io::Write;

use crate::value::Value;

pub type IdentifierId = usize;
pub type IdentifierName = String;
pub type LocalVarIndex = usize;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operation {
    Constant(IdentifierId),
    Nil,
    True,
    False,
    Pop,

    GetGlobal(IdentifierName),
    DefineGlobal(IdentifierName),
    SetGlobal(IdentifierName),

    GetLocal(LocalVarIndex),
    SetLocal(LocalVarIndex),

    Equal,
    Greater,
    Less,

    Add,
    Substract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,

    JumpIfFalse(usize),
    Loop(usize),
    Jump(usize),

    Call(u8),

    Return,
}

impl Operation {
    pub fn disassemble<W: Write>(
        &self,
        chunk: &Chunk,
        offset: usize,
        output: &mut W,
    ) -> core::result::Result<(), std::io::Error> {
        write!(output, "{:04} ", offset)?;
        if offset > 0 {
            write!(output, "   | ")?;
        }
        // else {
        // 	print!("{:4} ", chunk.lines[offset]);
        // }
        match self {
            Operation::Constant(constant_offset) => writeln!(
                output,
                "Constant	{} '{:?}'",
                constant_offset, &chunk.constants[*constant_offset]
            ),

            op => writeln!(output, "{:?}", op),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Chunk {
    pub code: Vec<Operation>,
    pub constants: Vec<Value>,
    lines: Vec<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: vec![],
            constants: vec![],
            lines: vec![],
        }
    }

    pub fn write(&mut self, op: Operation, line: u32) {
        self.code.push(op);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> IdentifierId {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn read_constant(&self, coffset: usize) -> &Value {
        &self.constants[coffset]
    }

    // pub fn disassemble(&self, name: &str) {
    //     println!("== {} ==", name);
    //     let mut offset: usize = 0;
    //     for op in &self.code {
    //         op.disassemble(self, offset);
    //         offset += 1;
    //     }
    // }

    pub fn emit(&mut self, op: Operation) {
        self.code.push(op);
    }

    pub fn emit_many(&mut self, ops: &mut Vec<Operation>) {
        self.code.append(ops);
    }

    pub fn emit_constant(&mut self, val: Value) {
        let constant = self.add_constant(val);
        self.emit(Operation::Constant(constant));
    }

    /// Get a reference to the chunk's code.
    pub fn code(&self) -> &[Operation] {
        self.code.as_ref()
    }

    pub fn op_count(&self) -> usize {
        self.code.len()
    }

    pub fn op_get(&self, offset: usize) -> Option<&Operation> {
        self.code.get(offset)
    }

    pub fn op_patch(&mut self, op_offset: usize, new_op: Operation) {
        self.code[op_offset] = new_op;
    }
}
