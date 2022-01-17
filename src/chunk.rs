use std::vec;

// use std::fmt::Display;

pub type Value = f64;

#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Constant(usize),

	Add, Substract, Multiply, Divide,
	Negate,
    
	Return,
}

impl Operation {
    pub fn disassemble(&self, chunk: &Chunk, offset: usize) {
        print!("{:04} ", offset);
		if offset > 0 && chunk.lines[offset] == chunk.lines[offset-1] {
			print!("   | ");
		} else {
			print!("{:4} ", chunk.lines[offset]);
		}
        match self {
            Operation::Constant(constant_offset) => {
                println!("Constant	{} '{}'", constant_offset, &chunk.constants[*constant_offset])
            }
            Operation::Add => println!("Add"),
            Operation::Substract => println!("Substract"),
            Operation::Multiply => println!("Multiply"),
            Operation::Divide => println!("Divide"),
            Operation::Negate => println!("Negate"),
            Operation::Return => println!("Return"),
        }
    }
}

// #[derive(Debug)]
pub struct Chunk {
    pub code: Vec<Operation>,
    pub constants: Vec<Value>,
	lines: Vec<u32>
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

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

	pub fn read_constant(&self, coffset: usize) -> Value {
		self.constants[coffset]
	}

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        let mut offset: usize = 0;
        for op in &self.code {
            op.disassemble(self, offset);
            offset += 1;
        }
    }

    pub fn emit(&mut self, op: Operation) {
        self.code.push(op);
    }

    pub fn emit_constant(&mut self, val: Value) {
        let constant = self.add_constant(val);
        self.emit(Operation::Constant(constant));
    }

    /// Get a reference to the chunk's code.
    pub fn code(&self) -> &[Operation] {
        self.code.as_ref()
    }
}

// impl Display for Chunk {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "=====", self.code)?;
//         write!(f, "{:#?}", self.code)
//     }
// }
