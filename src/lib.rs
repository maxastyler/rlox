#![feature(hash_set_entry)]

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

use vm::VM;

pub mod ast;
pub mod chunk;
pub mod compiler;
pub mod gc;
pub mod parser;
pub mod scanner;
pub mod value;
pub mod vm;

pub fn repl(mut vm: VM) {
    let mut buffer = String::with_capacity(1024);
    let stdin = io::stdin();
    loop {
        print!("> ");
        let handle = stdin.lock();
        buffer.clear();
        match handle.take(1024).read_line(&mut buffer) {
            Ok(0) | Err(_) => {
                println!();
                break;
            }
            _ => {}
        };
        // vm.interpret(&buffer);
    }
}

pub fn run_file(mut vm: VM, path: &str) {
    let file = File::open(path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    // vm.interpret(&contents);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
