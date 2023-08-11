use rlox::{
    chunk::{Chunk, OpCode},

};
use std::rc::Rc;

#[derive(Debug, Default)]
struct S(Vec<Rc<usize>>, Vec<Rc<usize>>);

impl S {
    fn add(&mut self, x: Vec<usize>) {
        for i in x {
            let S(v1, v2) = self;
            v1.push(Rc::new(i));
            v2.push(v1.last().unwrap().clone());
	    v1.last().unwrap().to_owned();
        }
    }
}

fn main() {
    let mut s = S::default();
    s.add(vec![2, 3, 4]);
    println!("{:?}", s);

    // let constant_ref = x.add_constant(1.2);
    // x.write_chunk(OpCode::OP_CONSTANT.into(), 123);
    // x.write_chunk(constant_ref as u8, 123);

    // let constant_ref = x.add_constant(3.4);
    // x.write_chunk(OpCode::OP_CONSTANT.into(), 123);
    // x.write_chunk(constant_ref as u8, 123);

    // x.write_chunk(OpCode::OP_ADD.into(), 123);

    // let constant_ref = x.add_constant(5.6);
    // x.write_chunk(OpCode::OP_CONSTANT.into(), 123);
    // x.write_chunk(constant_ref as u8, 123);

    // x.write_chunk(OpCode::OP_DIVIDE.into(), 123);

    // x.write_chunk(OpCode::OP_NEGATE.into(), 123);
    // x.write_chunk(OpCode::OP_RETURN.into(), 123);
    // disassemble_chunk(&x, "Cool chunk");
    // VM::new(x).interpret(&"1+2".into());
}
