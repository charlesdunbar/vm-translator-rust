#![allow(clippy::pedantic)]

use crate::parser::Parser;

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
    pointer: Vec<i16>,
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
            pointer: vec![0; 0x6000],
            temp: vec![0; 0x6000],
        }
    }

    fn string_to_vec(&self, str: &str) -> &Vec<i16> {
        match str {
            "argument" => return &self.argument,
            "local" => return &self.local,
            "static" => return &self.static_vec,
            "this" => return &self.this,
            "that" => return &self.that,
            "pointer" => return &self.pointer,
            "temp" => return &self.temp,
            _ => panic!("Error matching what vec to return!"),
        }
    }

    fn string_to_vec_mut(&mut self, str: &str, index: usize) -> Option<&mut i16> {
        match str {
            "argument" => return self.argument.get_mut(index),
            "local" => return self.local.get_mut(index),
            "static" => return self.static_vec.get_mut(index),
            "this" => return self.this.get_mut(index),
            "that" => return self.that.get_mut(index),
            "pointer" => return self.pointer.get_mut(index),
            "temp" => return self.temp.get_mut(index),
            _ => panic!("Error matching what vec to return!"),
        }
    }
}

pub struct CodeWriter<'a> {
    pub parser: Parser<'a>,
    stack: Vec<StackTypes>,
    memory: Memory,
}

impl<'a> CodeWriter<'a> {
    pub fn new(p: Parser<'a>) -> Self {
        CodeWriter {
            parser: p,
            stack: Vec::new(),
            memory: Memory::new(),
        }
    }

    pub fn write_arithmetic(&mut self) -> String {
        match self.parser.arg1().unwrap() {
            "add" => {
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(first + second));
                debug!("Stack is now {:?}", self.stack)
            }
            "sub" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(first - second));
                debug!("Stack is now {:?}", self.stack)
            }
            "neg" => {
                let StackTypes::Number(num) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(-num));
                debug!("Stack is now {:?}", self.stack)
            }
            "eq" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                if first == second {
                    self.stack.push(StackTypes::Number(TRUE));
                } else {
                    self.stack.push(StackTypes::Number(FALSE));
                }
                debug!("Stack is now {:?}", self.stack)
            }
            "gt" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                if first > second {
                    self.stack.push(StackTypes::Number(TRUE))
                } else {
                    self.stack.push(StackTypes::Number(FALSE))
                }
                debug!("Stack is now {:?}", self.stack)
            }
            "lt" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                if first < second {
                    self.stack.push(StackTypes::Number(TRUE))
                } else {
                    self.stack.push(StackTypes::Number(FALSE))
                }
                debug!("Stack is now {:?}", self.stack)
            }
            "and" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(first & second));
                debug!("Stack is now {:?}", self.stack)
            }
            "or" => {
                let StackTypes::Number(second) = self.stack.pop().unwrap();
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(first | second));
                debug!("Stack is now {:?}", self.stack)
            }
            "not" => {
                let StackTypes::Number(first) = self.stack.pop().unwrap();
                self.stack.push(StackTypes::Number(!first));
                debug!("Stack is now {:?}", self.stack)
            }
            _ => {
                panic!(
                    "Tried to do math on a not math ({:?}) thing!",
                    self.parser.arg1()
                )
            }
        }
        return format!("Math!\n");
    }

    pub fn write_push_pop(&mut self) -> String {
        match self.parser.current_command.split(' ').next() {
            Some(command) => match command {
                "push" => {
                    let segment = self.parser.arg1().unwrap();
                    let index = self.parser.clone().arg2().unwrap();
                    let mut write_string = formatdoc!("// push {segment} {index}\n");
                    let common_string = formatdoc!(
                        "@{index}
                        A=D+M
                        D=M
                        @SP
                        A=M
                        M=D
                        @SP
                        M=M+1
                        \n"
                    );
                    if segment == "constant" {
                        self.stack.push(index.into());
                        debug!("Stack is now {:?}", self.stack.clone());
                        write_string.push_str(&common_string);
                        return write_string
                    } else {
                        let memory = self.memory.string_to_vec(segment);
                        self.stack.push(StackTypes::Number(memory[index as usize]));

                        debug!("Stack is now {:?}", self.stack.clone());
                        let memory_string = formatdoc!("
                            @{segment}
                            D=M
                        ");
                        write_string.push_str(&memory_string);
                        write_string.push_str(&common_string);
                        return write_string
                    }
                }
                "pop" => {
                    let segment = self.parser.arg1().unwrap();
                    let index = self.parser.clone().arg2().unwrap();
                    let memory = self.memory.string_to_vec_mut(segment, index as usize);
                    let StackTypes::Number(i) = self.stack.pop().unwrap();
                    *memory.unwrap() = i;
                    return format!("pop");
                }
                _ => {
                    panic!("Error in matching what command to run in push_pop!")
                }
            },
            None => {
                panic!("Error in write_push_pop match!")
            }
        }
    }
}
