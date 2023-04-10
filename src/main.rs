#![allow(clippy::pedantic)]

mod code_writer;
mod parser;

use std::env;
use std::fs::File;
use std::io::Read;

use code_writer::CodeWriter;
use parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut file_contents = String::new();
    let mut file = File::open(&args[1]).expect("File not found");
    file.read_to_string(&mut file_contents)
        .expect("Could not read file");
    let p = Parser::new(&file_contents);
    let mut c = CodeWriter::new(p);
    //p.clone().print_lines();
    while c.parser.has_more_lines() {
        c.parser.advance();
        // println!(
        //     "{:?} {:?} {:?}",
        //     c.parser.command_type(),
        //     c.parser.arg1(),
        //     c.parser.clone().arg2()
        // );
        match c.parser.command_type() {
            parser::CommandType::ARITHMETIC => c.write_arithmetic(),
            parser::CommandType::PUSH | parser::CommandType::POP => c.write_push_pop(),
            _ => {}
        }
    }
    // let mut a = Assembler::new(&file_contents);
    // let to_write = a.generate_binary();
    // fs::write(
    //     format!("{}.hack", &args[1].split('.').nth(0).unwrap()),
    //     to_write.join("\n"),
    // )
    // .expect("Unable to write file");
}
