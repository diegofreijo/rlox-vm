use std::mem;

// use std::fmt::Display;


#[derive(Debug)]
pub enum Operation {
	Return,
}

impl Operation {
	pub fn disassemble(&self, offset: usize) {
		println!("{:04}	{:?}", offset, &self);
	}
}

// #[derive(Debug)]
pub struct Chunk {
	code: Vec<Operation>,
}

impl Chunk {
	pub fn new() -> Chunk {
		Chunk {
			code: vec![],
		}
	}

	pub fn write(&mut self, op: Operation) {
		self.code.push(op);
	}

	pub fn disassemble(&self, name: &str) {
		println!("== {} ==\n", name);
		let mut offset: usize = 0;
		for op in &self.code {
			op.disassemble(offset);
			offset += mem::size_of_val(&op);
		}
	}
}

// impl Display for Chunk {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "=====", self.code)?;
//         write!(f, "{:#?}", self.code)
//     }
// }
