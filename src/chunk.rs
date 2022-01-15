use std::vec;

// use std::fmt::Display;

type Value = f64;

#[derive(Debug)]
pub enum Operation {
    Constant(usize),
    Return,
}

impl Operation {
    pub fn disassemble(&self, chunk: &Chunk, offset: usize) {
        print!("{:04}  ", offset);
		if offset > 0 && chunk.lines[offset] == chunk.lines[offset-1] {
			print!("   | ");
		} else {
			print!("{:04} ", chunk.lines[offset]);
		}
        match self {
            Operation::Constant(constant_offset) => {
                println!("Constant {}", &chunk.constants[*constant_offset])
            }
            Operation::Return => println!("Return"),
        }
    }
}

// #[derive(Debug)]
pub struct Chunk {
    code: Vec<Operation>,
    constants: Vec<Value>,
	lines: Vec<u32>
}

impl Chunk {
    pub fn new() -> Chunk {
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

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==\n", name);
        let mut offset: usize = 0;
        for op in &self.code {
            op.disassemble(self, offset);
            offset += 1;
        }
    }
}

// impl Display for Chunk {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "=====", self.code)?;
//         write!(f, "{:#?}", self.code)
//     }
// }
