#![allow(clippy::pedantic)]

mod code_writer;
mod parser;

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{env};

use code_writer::CodeWriter;
use glob::glob;
use indoc::formatdoc;
use parser::Parser;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let mut file_contents = String::new();
    let f_or_d = PathBuf::from(&args[1]);
    let mut out_file: File;
    let filename: &str;
    if f_or_d.is_dir() {
        filename = &args[1];
        println!("Going to print to {}.asm", &args[1]);
        out_file = File::create(format!("{}/{}.asm", &args[1], filename)).expect("Unable to create new file");
        let mut c: CodeWriter = CodeWriter::new("Sys", true); // Boostrap code calls the Sys init function
        // Set up bootstrap code
        let bootstrap_code = formatdoc! {
            "@256
        D=A
        @SP
        M=D
        {}
        ", c.write_call("init", 0)
        };

        out_file
            .write(bootstrap_code.as_bytes())
            .expect("Error writing to file");

        for files in
            glob(format!("{}/*.vm", filename).as_str()).expect("Failed to read glob pattern")
        {
            match files {
                Ok(path) => {
                    println!("Parsing {}", path.display());
                    let mut file_contents = String::new();
                    let mut file = File::open(path.clone()).expect("Error opening file");
                    file.read_to_string(&mut file_contents)
                        .expect("Could not read file");
                    let mut p = Parser::new(&file_contents);
                    let mut c = CodeWriter::new(path.file_name().unwrap().to_str().unwrap().split('.').nth(0).unwrap(), false);

                    println!("{:?} has more lines: {:?}", &file_contents, p.has_more_lines());
                    while p.has_more_lines() {
                        p.advance();
                        match p.command_type() {
                            parser::CommandType::ARITHMETIC => out_file
                                .write(c.write_arithmetic(p.arg1().unwrap()).as_bytes())
                                .expect("Error writing to file"),
                            parser::CommandType::PUSH | parser::CommandType::POP => out_file
                                .write(c.write_push_pop(p.command_type(), p.arg1().unwrap(), p.clone().arg2().unwrap()).as_bytes())
                                .expect("Error writing to file"),
                            parser::CommandType::LABEL => out_file
                                .write(c.write_label(p.arg1().unwrap()).as_bytes())
                                .expect("Error writing to file"),
                            parser::CommandType::GOTO => out_file
                                .write(c.write_goto(p.arg1().unwrap()).as_bytes())
                                .expect("Error writing to file"),
                            parser::CommandType::IF => out_file
                                .write(c.write_if(p.arg1().unwrap()).as_bytes())
                                .expect("Error writing to file"),
                            parser::CommandType::FUNCTION => out_file
                                .write(c.write_function(p.arg1().unwrap(), p.clone().arg2().unwrap()).as_bytes())
                                .expect("Error writing to file"),
                            parser::CommandType::RETURN => out_file
                                .write(c.write_return().as_bytes())
                                .expect("Error writing to file"),
                            parser::CommandType::CALL =>                             
                                out_file
                                .write(c.write_call(p.arg1().unwrap(), p.clone().arg2().unwrap()).as_bytes())
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
                Err(e) => println!("{:?}", e),
            } //let mut file = File::open(&args[1]).expect("File not found");
        }
    } else {
        let mut file = File::open(&args[1]).expect("File not found");
        file.read_to_string(&mut file_contents)
            .expect("Could not read file");
        let mut p = Parser::new(&file_contents);
        let file_str = format!("{}", &args[1].split('.').nth(0).unwrap());
        filename = file_str.as_str();
        let mut c: CodeWriter = CodeWriter::new(&filename, true);
        out_file = File::create(format!("{}.asm", &filename)).expect("Unable to create new file");

        // Set up bootstrap code
        let bootstrap_code = formatdoc! {
            "@256
        D=A
        @SP
        M=D
        {}
        ", c.write_call("init", 0)
        };
        out_file
            .write(bootstrap_code.as_bytes())
            .expect("Error writing to file");

            while p.has_more_lines() {
                p.advance();
                match p.command_type() {
                    parser::CommandType::ARITHMETIC => out_file
                        .write(c.write_arithmetic(p.arg1().unwrap()).as_bytes())
                        .expect("Error writing to file"),
                    parser::CommandType::PUSH | parser::CommandType::POP => out_file
                        .write(c.write_push_pop(p.command_type(), p.arg1().unwrap(), p.clone().arg2().unwrap()).as_bytes())
                        .expect("Error writing to file"),
                    parser::CommandType::LABEL => out_file
                        .write(c.write_label(p.arg1().unwrap()).as_bytes())
                        .expect("Error writing to file"),
                    parser::CommandType::GOTO => out_file
                        .write(c.write_goto(p.arg1().unwrap()).as_bytes())
                        .expect("Error writing to file"),
                    parser::CommandType::IF => out_file
                        .write(c.write_if(p.arg1().unwrap()).as_bytes())
                        .expect("Error writing to file"),
                    parser::CommandType::FUNCTION => out_file
                        .write(c.write_function(p.arg1().unwrap(), p.clone().arg2().unwrap()).as_bytes())
                        .expect("Error writing to file"),
                    parser::CommandType::RETURN => out_file
                        .write(c.write_return().as_bytes())
                        .expect("Error writing to file"),
                    parser::CommandType::CALL =>                             
                        out_file
                        .write(c.write_call(p.arg1().unwrap(), p.clone().arg2().unwrap()).as_bytes())
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
}
