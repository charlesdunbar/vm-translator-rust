#![allow(clippy::pedantic)]

use log::info;
use std::str::Lines;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum CommandType {
    ARITHMETIC,
    PUSH,
    POP,
    LABEL,
    GOTO,
    IF,
    FUNCTION,
    RETURN,
    CALL,
}

#[derive(Clone)]
pub struct Parser<'a> {
    pub current_line: u16,
    source_iterator: Lines<'a>,
    pub current_command: String,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser {
            current_line: 0,
            source_iterator: input.lines(),
            current_command: String::from(""),
        }
    }

    /// Are there more lines in the input?
    pub fn has_more_lines(&self) -> bool {
        if let Some(_) = self.source_iterator.clone().peekable().peek() {
            true
        } else {
            false
        }
    }

    /// Reads the next command from the input and makes it the current command.
    ///
    /// Exits early if has_more_lines() is false.
    pub fn advance(&mut self) {
        if !self.has_more_lines() {
            return;
        }
        match self.source_iterator.next() {
            Some(line) => match line.split_whitespace().next() {
                Some(command) => {
                    //TODO: Skip over blanks and comments.
                    match command.chars().next() {
                        Some(c) => {
                            match c {
                                '/' => {
                                    if line.chars().next() == Some('/') {
                                        // Do nothing to skip over comments
                                        self.advance()
                                    }
                                }
                                _ => {
                                    info!("{}", line);
                                    self.current_command = String::from(line)
                                }
                            }
                        }
                        None => {
                            // Skip blank lines
                            self.advance()
                        }
                    }
                }
                None => return,
            },
            None => {
                self.current_command = String::from("");
                println!("No Match!");
            }
        }
    }

    /// Returns a representation of the current command.
    ///
    /// If the current command is an arithmetic-logical command, returns C_ARITHMETIC
    pub fn command_type(&self) -> CommandType {
        match self.current_command.split_whitespace().next() {
            None => {
                panic!("Tried to get the command type of nothing!")
            }
            Some(command) => match command {
                "push" => CommandType::PUSH,
                "pop" => CommandType::POP,
                "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => {
                    CommandType::ARITHMETIC
                }
                "label" => CommandType::LABEL,
                "goto" => CommandType::GOTO,
                "if-goto" => CommandType::IF,
                "function" => CommandType::FUNCTION,
                "call" => CommandType::CALL,
                "return" => CommandType::RETURN,
                _ => panic!("reached unreachable branch of command_type match"),
            },
        }
    }

    /// Returns the first argument of the current command.
    ///
    /// In the case of C_ARITHMETIC, the command itself (add, sub, etc.) is returned.
    ///
    /// Exits early if the current command is C_RETURN.
    pub fn arg1(&self) -> Option<&str> {
        let mut ret_val = self.current_command.split_whitespace();
        if let CommandType::RETURN = self.command_type() {
            return None;
        } else if let CommandType::ARITHMETIC = self.command_type() {
            return ret_val.next();
        } else {
            return self.current_command.split_whitespace().nth(1);
        }
    }

    /// Returns the second argument of the current command.
    ///
    /// Exits early if command is not C_PUSH, C_POP, C_FUNCTION, or C_CALL
    pub fn arg2(self) -> Option<i16> {
        match self.command_type() {
            CommandType::PUSH | CommandType::POP | CommandType::FUNCTION | CommandType::CALL => {
                return Some(
                    self.current_command
                        .split_whitespace()
                        .nth(2)?
                        .parse::<i16>()
                        .unwrap(),
                )
            }
            _ => return None,
        }
    }
}
