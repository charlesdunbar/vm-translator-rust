#![allow(clippy::pedantic)]

use crate::parser::Parser;

use log::debug;

#[derive(Debug, Clone)]
enum StackTypes {
    Eq(bool),
    Number(usize),
}

impl From<usize> for StackTypes {
    fn from(value: usize) -> Self {
        StackTypes::Number(value)
    }
}

impl From<bool> for StackTypes {
    fn from(value: bool) -> Self {
        StackTypes::Eq(value)
    }
}


struct Memory {
    argument: Vec<usize>,
    local: Vec<usize>,
    static_vec: Vec<usize>,
    this: Vec<usize>,
    that: Vec<usize>,
    pointer: Vec<usize>,
    temp: Vec<usize>,
}

impl Memory {
    fn new() -> Self {
        Memory {
            argument: Vec::new(),
            local: Vec::new(),
            static_vec: Vec::new(),
            this: Vec::new(),
            that: Vec::new(),
            pointer: Vec::new(),
            temp: Vec::new(),
        }
    }

    fn string_to_vec(&self, str: &str) -> &Vec<usize> {
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

    fn string_to_vec_mut(&mut self, str: &str, index: usize) -> Option<&mut usize> {
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

    pub fn write_arithmetic(&mut self) {
        match self.parser.arg1().unwrap() {
            "add" => {
                if let StackTypes::Number(val) = self.stack.pop().unwrap() {
                    let first = val;
                    if let StackTypes::Number(val) = self.stack.pop().unwrap() {
                        let second = val;
                        self.stack.push(StackTypes::Number(first + second));
                    }

                }
                debug!("Stack is now {:?}", self.stack)
            }
            "sub" => {}
            "neg" => {}
            "eq" => {
                // let first = self.stack.pop().unwrap();
                // let second = self.stack.pop().unwrap();
                // let eq = first.into() == second.into();
                // self.stack.push(StackTypes::Eq(eq));
            }
            "gt" => {}
            "lt" => {}
            "and" => {}
            "or" => {}
            "not" => {}
            _ => {
                panic!(
                    "Tried to do math on a not math ({:?}) thing!",
                    self.parser.arg1()
                )
            }
        }
    }

    pub fn write_push_pop(&mut self) {
        match self.parser.current_command.split(' ').next() {
            Some(command) => match command {
                "push" => {
                    let segment = self.parser.arg1().unwrap();
                    let index = self.parser.clone().arg2().unwrap();
                    if segment == "constant" {
                        self.stack.push(index.into())
                    } else {
                        let memory = self.memory.string_to_vec(segment);
                        self.stack.push(StackTypes::Number(memory[index]));
                    }
                    debug!("Stack is now {:?}", self.stack.clone())
                }
                "pop" => {
                    let segment = self.parser.arg1().unwrap();
                    let index = self.parser.clone().arg2().unwrap();
                    let memory = self.memory.string_to_vec_mut(segment, index);
                    if let StackTypes::Number(i) = self.stack.pop().unwrap() {
                        *memory.unwrap() = i
                    }
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
