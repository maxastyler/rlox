use crate::{
    chunk::{Chunk, OpCode},
    value::print_value,
};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);
    let mut offset = 0;
    while offset < chunk.codes.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ")
    } else {
        print!("{:>4} ", chunk.lines[offset])
    }
    let instruction = chunk.codes[offset];
    if let Ok(c) = OpCode::try_from(instruction) {
        match c {
            OpCode::OP_RETURN => simple_instruction("OP_RETURN", offset),
            OpCode::OP_NEGATE => simple_instruction("OP_NEGATE", offset),
            OpCode::OP_CONSTANT => constant_instruction("OP_CONSTANT", chunk, offset),
	    OpCode::OP_ADD => simple_instruction("OP_ADD", offset),
	    OpCode::OP_MULTIPLY => simple_instruction("OP_MULTIPLY", offset),
	    OpCode::OP_DIVIDE => simple_instruction("OP_DIVIDE", offset),
	    OpCode::OP_SUBTRACT => simple_instruction("OP_SUBTRACT", offset),
        }
    } else {
        println!("Unknown opcode {}", instruction);
        offset + 1
    }
}

fn simple_instruction(text: &str, offset: usize) -> usize {
    println!("{}", text);
    offset + 1
}

fn constant_instruction(text: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant_loc = chunk.codes[offset + 1];
    let constant = chunk.values[constant_loc as usize];
    print!("{:<16} {:>4} '", text, constant_loc);
    print_value(&constant);
    println!("'");
    offset + 2
}
