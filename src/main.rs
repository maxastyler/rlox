use rlox::{
    chunk::{Chunk, OpCode},
    debug::disassemble_chunk,
};

fn main() {
    let mut x = Chunk::default();
    let constant_ref = x.add_constant(1.2);
    x.write_chunk(OpCode::OP_CONSTANT.into(), 123);
    x.write_chunk(constant_ref as u8, 123);
    x.write_chunk(OpCode::OP_RETURN.into(), 123);
    disassemble_chunk(&x, "Cool chunk");
}
