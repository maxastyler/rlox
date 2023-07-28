use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

use vm::VM;

pub mod chunk;
pub mod compile;
pub mod debug;
pub mod scanner;
pub mod value;
pub mod vm;

pub fn repl(vm: VM) {
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
        // vm.interpret(buffer);
    }
}

pub fn run_file(vm: VM, path: &str) {
    let file = File::open(path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    // vm.interpret(contents);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
