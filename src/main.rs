#![allow(clippy::pedantic)]

mod code_writer;
mod parser;

use std::env;
use std::fs::File;
use std::io::{Read, Write};

use code_writer::CodeWriter;
use indoc::formatdoc;
use parser::Parser;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let mut file_contents = String::new();
    let mut file = File::open(&args[1]).expect("File not found");
    file.read_to_string(&mut file_contents)
        .expect("Could not read file");
    let p = Parser::new(&file_contents);
    let filename = format!("{}", &args[1].split('.').nth(0).unwrap());
    let mut c = CodeWriter::new(p, &filename);
    let mut out_file =
        File::create(format!("{}.asm", &filename)).expect("Unable to create new file");

    while c.parser.has_more_lines() {
        c.parser.advance();
        match c.parser.command_type() {
            parser::CommandType::ARITHMETIC => out_file
                .write(c.write_arithmetic().as_bytes())
                .expect("Error writing to file"),
            parser::CommandType::PUSH | parser::CommandType::POP => out_file
                .write(c.write_push_pop().as_bytes())
                .expect("Error writing to file"),
            _ => out_file
                .write(String::from("Need to Implement").as_bytes())
                .expect("Error writing to file"),
        };
    }
    // Finish program with infinite loop
    let infinite_loop = formatdoc! {"
        (INFINITE_LOOP)
        @INFINITE_LOOP
        0;JMP            // infinite loop
    "};
    out_file
        .write(infinite_loop.as_bytes())
        .expect("Error writing to file");
}
