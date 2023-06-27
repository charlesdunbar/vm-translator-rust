#![allow(clippy::pedantic)]

use crate::parser::CommandType;
use std::collections::HashMap;

use log::info;

use indoc::formatdoc;

const TRUE: i16 = -1;
const FALSE: i16 = 0;

pub struct CodeWriter<'a> {
    filename: &'a str,
    op_lookup: HashMap<String, String>,
    memory_lookup: HashMap<String, String>,
    jmp_counter: i16,
    call_counter: &'a mut i16,
    current_function: String,
}

impl<'a> CodeWriter<'a> {
    pub fn new(filename: &'a str, call_counter: &'a mut i16) -> Self {
        CodeWriter {
            filename,
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
            call_counter,
            current_function: String::from("bootstrap"),
        }
    }

    pub fn write_label(&self, label: &str) -> String {
        // TODO - Handle multiple files and knowing if inside a function when called.
        // Track current function in code_writer
        let write_string = formatdoc! {
            "
            // label {label}
            ({}.{}${label})
            ", self.filename, self.current_function
        };
        write_string
    }

    pub fn write_goto(&self, label: &str) -> String {
        let write_string = formatdoc! {
            "
            // goto {}.{}${label}
            @{}.{}${label}
            0;JMP

            ", self.filename, self.current_function, self.filename, self.current_function
        };
        write_string
    }

    pub fn write_if(&self, label: &str) -> String {
        let write_string = formatdoc! {
            "
            // if-goto {label}
            {}
            @{}.{}${label}
            D;JNE

            ", self.generate_pop_stack(true), self.filename, self.current_function
        };
        write_string
    }

    pub fn write_function(&mut self, function_name: &str, n_vars: i16) -> String {
        let mut write_string = formatdoc! {
            "
            // function {function_name} {n_vars}
            ({function_name})
            "
        };
        for _ in 0..n_vars {
            write_string.push_str(
                formatdoc! {"
            @0
            D=A
            @SP
            A=M
            M=D
            @SP
            M=M+1
            "}
                .as_str(),
            );
        }
        self.current_function = String::from(function_name.split('.').nth(1).unwrap());
        write_string.push_str("\n");
        write_string
    }

    pub fn write_call(&mut self, function_name: &str, n_args: i16) -> String {
        info!("function_name in call is {:?}", function_name);
        *self.call_counter += 1;
        let write_string = formatdoc! {
            // TODO - Call other functions to improve this
            "// call {function_name} {n_args}
            // Generate return address label and push to stack
            @{function_name}$ret.{}
            D=A
            @SP
            A=M
            M=D
            @SP
            M=M+1
            // Push LCL
            @LCL
            D=M
            @SP
            A=M
            M=D
            @SP
            M=M+1
            // Push ARG
            @ARG
            D=M
            @SP
            A=M
            M=D
            @SP
            M=M+1
            // Push THIS
            @THIS
            D=M
            @SP
            A=M
            M=D
            @SP
            M=M+1
            // Push THAT
            @THAT
            D=M
            @SP
            A=M
            M=D
            @SP
            M=M+1
            // ARG = SP-5-nArgs
            @5
            D=A
            @SP
            D=M-D
            @{n_args}
            D=D-A
            @ARG
            M=D
            // LCL = SP
            @SP
            D=M
            @LCL
            M=D
            // goto {function_name}
            @{function_name}
            0;JMP
            ({function_name}$ret.{})
            
            ", self.call_counter, self.call_counter,
        };
        info!(
            "after call statement, call_counter is now : {:?}",
            self.call_counter
        );
        write_string
    }

    pub fn write_return(&mut self) -> String {
        // TODO: Call other functions to improve this.
        let write_string = formatdoc! {
            "// return
            // Store LCL in frame
            @LCL
            D=M
            @FRAME
            M=D
            // Store retAddress *(frame-5) in @RET
            @5
            D=D-A
            @RET
            AM=D
            D=M
            @RET
            M=D
            // Pop the return value for caller
            {}
            @ARG
            A=M
            M=D
            // Restore caller's SP (ARG+1)
            @ARG
            D=M
            D=D+1
            @SP
            M=D
            // Restore THAT for caller *(frame-1)
            @FRAME
            D=M
            A=M-1
            D=M
            @THAT
            M=D
            // Restore THIS for caller *(frame-2)
            @2
            D=A
            @FRAME
            D=M-D
            A=D
            D=M
            @THIS
            M=D
            // Restore ARG for caller *(frame-3)
            @3
            D=A
            @FRAME
            D=M-D
            A=D
            D=M
            @ARG
            M=D
            // Restore LCL for caller *(frame-4)
            @4
            D=A
            @FRAME
            D=M-D
            A=D
            D=M
            @LCL
            M=D
            // goto return address
            @RET
            A=M
            0;JMP
            
            ", self.generate_pop_stack(true)
        };

        write_string
    }

    pub fn write_arithmetic(&mut self, op: &str) -> String {
        match op {
            "add" | "sub" | "and" | "or" => {
                self.generate_math_string(String::from(op), false, None)
            }
            "neg" | "not" => self.generate_math_string(String::from(op), true, None),
            "eq" => self.generate_math_string(String::from("eq"), false, Some(String::from("JEQ"))),
            "gt" => self.generate_math_string(String::from("gt"), false, Some(String::from("JGT"))),
            "lt" => self.generate_math_string(String::from("lt"), false, Some(String::from("JLT"))),
            _ => {
                panic!("Tried to do math on a not math ({:?}) thing!", op)
            }
        }
    }

    pub fn write_push_pop(&self, command_type: CommandType, segment: &str, index: i16) -> String {
        match command_type {
            // TODO: move segment and index matching here, pass to push and pop
            CommandType::PUSH => self.generate_push_string(segment, index),
            CommandType::POP => self.generate_pop_string(segment, index),
            _ => {
                panic!("Error in matching what command to run in push_pop!")
            }
        }
    }

    /// Generate a string of hack asm to pop the value off the stack
    ///
    /// # Arguments
    /// * `store_d` - if true, store the popped value in D
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
        write_string
    }

    fn generate_push_string(&self, segment: &str, index: i16) -> String {
        let comment_string = format!("// push {segment} {index}");
        let common_string = formatdoc!(
            "@SP
            A=M // Go to Stack pointer
            M=D // Set RAM[SP] equal to D"
        );
        // constant doesn't need to store in any memory
        if segment == "constant" {
            let write_string = formatdoc! {
                "{}
                 @{index}
                 D=A
                 {}", comment_string, common_string
            };
            increment_stack_pointer(&write_string)
        } else if segment == "static" {
            let write_string = formatdoc! {
                "{comment_string}
                @{}.{index}
                D=M
                {common_string}", self.filename
            };
            increment_stack_pointer(&write_string)
        } else if segment == "temp" {
            let write_string = formatdoc! {
                "{comment_string}
                @{}
                D=M
                {}", index + 5, common_string
            };
            increment_stack_pointer(&write_string)
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
            increment_stack_pointer(&write_string)
        }
    }

    fn generate_pop_string(&self, segment: &str, index: i16) -> String {
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
            common_string
        } else if segment == "temp" {
            let common_string = formatdoc! {
                "{comment_string}
                {}
                @{}
                M=D
                
                ", self.generate_pop_stack(true), index + 5
            };
            common_string
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

            // A=D+A
            // D=A // D contains RAM + Offset
            // equals
            //
            // AD=D+A
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
            common_string
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
        increment_stack_pointer(&common_string)
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
    formatdoc!("{}{}", command, to_append)
}
