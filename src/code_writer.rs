#![allow(clippy::pedantic)]

use crate::parser::{CommandType, Parser};
use std::collections::HashMap;

use indoc::formatdoc;
use log::debug;

const TRUE: i16 = -1;
const FALSE: i16 = 0;

#[derive(Debug, Clone)]
enum StackTypes {
    Number(i16),
}

impl From<i16> for StackTypes {
    fn from(value: i16) -> Self {
        StackTypes::Number(value)
    }
}

struct Memory {
    argument: Vec<i16>,
    local: Vec<i16>,
    static_vec: Vec<i16>,
    this: Vec<i16>,
    that: Vec<i16>,
    temp: Vec<i16>,
}

impl Memory {
    fn new() -> Self {
        Memory {
            argument: vec![0; 0x6000],
            local: vec![0; 0x6000],
            static_vec: vec![0; 0x6000],
            this: vec![0; 0x6000],
            that: vec![0; 0x6000],
            temp: vec![0; 0x6000],
        }
    }

    /// With a string input, return the vec memory that has the same name.
    ///
    /// Valid options are "argument", "local", "static", "this", "that", "pointer", and "temp"
    fn string_to_vec(&self, str: &str) -> &Vec<i16> {
        match str {
            "argument" => return &self.argument,
            "local" => return &self.local,
            "static" => return &self.static_vec,
            "this" => return &self.this,
            "that" => return &self.that,
            "temp" => return &self.temp,
            _ => panic!("Error matching what vec to return!"),
        }
    }

    /// With a string input, return the mut vec memory that has the same name at the specific index.
    ///
    /// Valid options are "argument", "local", "static", "this", "that", "pointer", and "temp"
    fn string_to_vec_mut(&mut self, str: &str, index: usize) -> Option<&mut i16> {
        match str {
            "argument" => return self.argument.get_mut(index),
            "local" => return self.local.get_mut(index),
            "static" => return self.static_vec.get_mut(index),
            "this" => return self.this.get_mut(index),
            "that" => return self.that.get_mut(index),
            "temp" => return self.temp.get_mut(index),
            _ => panic!("Error matching what vec to return!"),
        }
    }
}

pub struct CodeWriter<'a> {
    filename: &'a str,
    pub parser: Parser<'a>,
    stack: Vec<StackTypes>,
    memory: Memory,
    op_lookup: HashMap<String, String>,
    memory_lookup: HashMap<String, String>,
    jmp_counter: i16,
}

impl<'a> CodeWriter<'a> {
    pub fn new(p: Parser<'a>, filename: &'a str) -> Self {
        CodeWriter {
            filename,
            parser: p,
            stack: Vec::new(),
            memory: Memory::new(),
            op_lookup: HashMap::from([
                (String::from("add"), String::from("+")),
                (String::from("sub"), String::from("-")),
                (String::from("neg"), String::from("-")),
                // Next 3 only care about jump insruction after the math
                (String::from("eq"), String::from("-")),
                (String::from("gt"), String::from("-")),
                (String::from("lt"), String::from("-")),
                (String::from("and"), String::from("&")),
                (String::from("or"), String::from("|")),
                (String::from("not"), String::from("!")),
            ]),
            memory_lookup: HashMap::from([
                (String::from("local"), String::from("LCL")),
                (String::from("argument"), String::from("ARG")),
                (String::from("this"), String::from("THIS")),
                (String::from("that"), String::from("THAT")),
                (String::from("temp"), String::from("TEMP")),
            ]),
            jmp_counter: 0,
        }
    }

    pub fn write_arithmetic(&mut self) -> String {
        match self.parser.arg1().unwrap() {
            "add" => {
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let total = first + second;
                self.stack.push(StackTypes::Number(total));
                debug!("Stack is now {:?}", self.stack);

                return self.generate_math_string(String::from("add"), false, None);
            }
            "sub" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(first - second));
                debug!("Stack is now {:?}", self.stack);

                return self.generate_math_string(String::from("sub"), false, None);
            }
            "neg" => {
                let StackTypes::Number(num) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(-num));
                debug!("Stack is now {:?}", self.stack);

                return self.generate_math_string(String::from("neg"), true, None);
            }
            "eq" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                if first == second {
                    self.stack.push(StackTypes::Number(TRUE));
                } else {
                    self.stack.push(StackTypes::Number(FALSE));
                }
                debug!("Stack is now {:?}", self.stack);

                return self.generate_math_string(
                    String::from("eq"),
                    false,
                    Some(String::from("JEQ")),
                );
            }
            "gt" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                if first > second {
                    self.stack.push(StackTypes::Number(TRUE))
                } else {
                    self.stack.push(StackTypes::Number(FALSE))
                }
                debug!("Stack is now {:?}", self.stack);

                return self.generate_math_string(
                    String::from("gt"),
                    false,
                    Some(String::from("JGT")),
                );
            }
            "lt" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                if first < second {
                    self.stack.push(StackTypes::Number(TRUE))
                } else {
                    self.stack.push(StackTypes::Number(FALSE))
                }
                debug!("Stack is now {:?}", self.stack);

                return self.generate_math_string(
                    String::from("lt"),
                    false,
                    Some(String::from("JLT")),
                );
            }
            "and" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(first & second));
                debug!("Stack is now {:?}", self.stack);

                return self.generate_math_string(String::from("and"), false, None);
            }
            "or" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(first | second));
                debug!("Stack is now {:?}", self.stack);

                return self.generate_math_string(String::from("or"), false, None);
            }
            "not" => {
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(!first));
                debug!("Stack is now {:?}", self.stack);

                return self.generate_math_string(String::from("not"), true, None);
            }
            _ => {
                panic!(
                    "Tried to do math on a not math ({:?}) thing!",
                    self.parser.arg1()
                )
            }
        }
        //return format!("Math!\n");
    }

    pub fn write_push_pop(&mut self) -> String {
        match self.parser.command_type() {
            // TODO: move segment and index matching here, pass to push and pop
            CommandType::PUSH => self.generate_push_string(),
            CommandType::POP => self.generate_pop_string(),
            _ => {
                panic!("Error in matching what command to run in push_pop!")
            }
        }
    }

    fn generate_pop_stack(&self, store_d: bool) -> String {
        // AM=M-1 is the shorter version of
        //
        // M=M=1
        // A=M
        let write_string = formatdoc! {
            "
            @SP
            AM=M-1"
        };
        if store_d {
            return formatdoc! {"
            {}
            D=M // Grab element-- from memory", write_string};
        }
        return write_string;
    }

    fn generate_push_string(&mut self) -> String {
        let segment = self.parser.arg1().unwrap();
        let index = self.parser.clone().arg2().unwrap();
        let comment_string = format!("// push {segment} {index}");
        let common_string = formatdoc!(
            "@SP
            A=M // Go to Stack pointer
            M=D // Set RAM[SP] equal to D"
        );
        // constant doesn't need to store in any memory
        if segment == "constant" {
            self.stack.push(index.into());
            debug!("Stack is now {:?}", self.stack.clone());
            let write_string = formatdoc! {
                "{}
                 @{index}
                 D=A
                 {}", comment_string, common_string
            };
            return increment_stack_pointer(&write_string);
        } else if segment == "static" {
            self.stack.push(index.into());
            debug!("Stack is now {:?}", self.stack.clone());
            let write_string = formatdoc! {
                "{comment_string}
                @{}.{index}

                D=M
                {common_string}", self.filename
            };
            return increment_stack_pointer(&write_string);
        } else if segment == "temp" {
            self.stack.push(index.into());
            debug!("Stack is now {:?}", self.stack.clone());
            let write_string = formatdoc! {
                "{comment_string}
                @{}
                D=M
                {}", index + 5, common_string
            };
            return increment_stack_pointer(&write_string);
        } else {
            // pointer 0 == THIS
            // pointer 1 == THAT
            // push pointer 0 pushes THIS's value to the stack
            if segment == "pointer" {
                let ptr_segment: &str;
                if index == 0 {
                    ptr_segment = "this";
                } else if index == 1 {
                    ptr_segment = "that";
                } else {
                    panic!("pointer can only be 0 or 1!");
                }
                let write_string = formatdoc! {
                    "{comment_string}
                    @{}
                    D=M
                    {common_string}", self.memory_lookup[ptr_segment]
                };

                return increment_stack_pointer(&write_string);
            }

            let memory = self.memory.string_to_vec(segment);
            self.stack.push(StackTypes::Number(memory[index as usize]));

            debug!("Stack is now {:?}", self.stack.clone());
            debug!("{segment} is now {:?}", memory);
            let write_string = formatdoc!(
                "{comment_string}
                @{}
                D=M // Store RAM location
                @{index}
                A=D+A // Go to RAM + Offset
                D=M // Get RAM[index] in D
                {common_string}",
                self.memory_lookup[segment],
            );
            return increment_stack_pointer(&write_string);
        }
    }

    fn generate_pop_string(&mut self) -> String {
        let segment = self.parser.arg1().unwrap();
        let index = self.parser.clone().arg2().unwrap();

        let comment_string = format!("// pop {segment} {index}");

        if segment == "constant" {
            panic!("Can't pop constant!")
        } else if segment == "static" {
            let common_string = formatdoc! {
                "{comment_string}
                {}
                @{}.{index}
                M=D
                
                ", self.generate_pop_stack(true), self.filename
            };
            return common_string;
        } else if segment == "temp" {
            let common_string = formatdoc! {
                "{comment_string}
                {}
                @{}
                M=D
                
                ", self.generate_pop_stack(true), index + 5
            };
            return common_string;
        } else {
            // pop pointer 0 sets THIS's memory to the stack value
            if segment == "pointer" {
                let ptr_segment: &str;
                if index == 0 {
                    ptr_segment = "this";
                } else if index == 1 {
                    ptr_segment = "that";
                } else {
                    panic!("pointer can only be 0 or 1!");
                }
                let common_string = formatdoc! {
                    "{comment_string}
                    {}
                    @{}
                    M=D

                    ", self.generate_pop_stack(true), self.memory_lookup[ptr_segment]
                };
                return common_string;
            }
            let memory = self.memory.string_to_vec_mut(segment, index as usize);
            let StackTypes::Number(i) = self.stack.pop().unwrap();
            *memory.unwrap() = i;

            debug!("Stack is now {:?}", self.stack.clone());
            debug!("{segment} is now {:?}", self.memory.string_to_vec(segment));

            // A=D+A
            // D=A // D contains RAM + Offset
            // equals
            //
            //AD=D+A
            let common_string = formatdoc! {
                "{comment_string}
                @{}
                D=M
                @{index}
                AD=D+A
                @R13
                M=D // Temp store RAM + Offset
                {}
                @R13
                A=M // Jump to RAM + Offset
                M=D
                
                ", self.memory_lookup[segment], self.generate_pop_stack(true)
            };
            return common_string;
        }
    }

    /// Generate a string of commands to update
    /// @SP-2 with the calculated math operation that was performed.
    fn generate_math_string(&mut self, op: String, unary: bool, jump: Option<String>) -> String {
        let mut common_string = formatdoc! {
            "// {op}
            {}
            ", self.generate_pop_stack(true)
        };
        if !unary {
            common_string.push_str(self.generate_pop_stack(false).as_str());
            common_string.push_str("\n");
        }
        match jump {
            Some(_) => common_string.push_str("D="),
            _ => common_string.push_str("M="),
        } // D if jump, M if math.
        if !unary {
            common_string.push_str("M");
        }
        common_string.push_str(format!("{}D", self.op_lookup[&op]).as_str());
        match jump {
            Some(j) => common_string.push_str(self.generate_jump_string(j).as_str()),
            None => {}
        }
        return increment_stack_pointer(&common_string);
    }

    fn generate_jump_string(&mut self, jump: String) -> String {
        let common_string = formatdoc! {
            "
            
            @TRUE_{}
            D;{jump}
            @SP
            A=M
            M={FALSE}
            @FALSE_{}
            0;JMP
            (TRUE_{})
            @SP
            A=M
            M={TRUE}
            (FALSE_{})", self.jmp_counter, self.jmp_counter, self.jmp_counter, self.jmp_counter
        };
        self.jmp_counter += 1;
        common_string
    }
}

/// Append the opcodes to increment the stack pointer to the command input.
fn increment_stack_pointer(command: &String) -> String {
    let to_append = formatdoc!(
        "
    
    @SP
    M=M+1

    "
    );
    return formatdoc!("{}{}", command, to_append);
}
