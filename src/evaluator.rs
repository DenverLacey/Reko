use crate::compiler;
use crate::string;

#[derive(Debug)]
pub struct Function {
    pub code: compiler::Code,
}

impl Function {
    pub fn new() -> Self {
        Self {
            code: compiler::Code::new(),
        }
    }
}

#[derive(Debug)]
pub struct Program {
    entry_index: usize,
    pub variable_size: usize,
    pub functions: Vec<Function>,
    // _strings: Vec<String>,
    strings: Vec<Box<[u8]>>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            entry_index: 0,
            variable_size: 0,
            functions: Vec::new(),
            // _strings: Vec::new(),
            strings: Vec::new(),
        }
    }

    pub fn set_entry_index(&mut self, entry_index: usize) {
        self.entry_index = entry_index;
    }

    pub fn add_string_constant(&mut self, string: String) -> Result<usize, String> {
        if let Some(index) = self
            .strings
            .iter()
            .position(|s| &s[..s.len() - 1] == string.as_bytes())
        {
            Ok(index)
        } else {
            let zstring = string::make_from_str(&string)
                .map_err(|err| format!("Failed to allocate string constant: {err}"))?;
            self.strings.push(zstring);
            Ok(self.strings.len() - 1)
        }
    }
}

// Key:
// () = arguments in the code
// [] = arguments on the data stack
// {} = arguments on the bind stack
// -a = peek argument (doesn't pop)
//
#[derive(Debug)]
pub enum Instruction {
    _NoOp, // 0. Just to reserve 0

    PushBool, // 1. (a) -> [a]
    PushInt,  // 2. (a) -> [a]
    PushStr,  // 3. (index in string table) -> [size, ptr]

    Dup,  // 4. [-a] -> [a, a]
    Over, // 5. [-a, -b] -> [a, b, a]
    Drop, // 6. [a] -> []
    Swap, // 7. [a, b] -> [b, a]

    PrintBool, // 8. [a] -> []
    PrintInt,  // 9. [a] -> []
    PrintStr,  // 10. [size, ptr] -> []

    Call,   // 11. (fid) -> [return values]
    Return, // 12. [] -> []

    And,      // 13. [a, b] -> [c]
    Or,       // 14. [a, b] -> [c]
    Not,      // 15. [a] -> [a]
    Add,      // 16. [a, b] -> [c]
    Subtract, // 17. [a, b] -> [c]
    Multiply, // 18. [a, b] -> [c]
    Divide,   // 19. [a, b] -> [c]

    Eq,      // 20. [a, b] -> [c]
    Neq,     // 21. [a, b] -> [c]
    Lt,      // 22. [a, b] -> [c]
    Gt,      // 23. [a, b] -> [c]
    Assign,  // 24. [ptr, a] -> []
    Load,    // 25. [ptr] -> [a]
    LoadStr, // 26. [ptr] -> [size, chars]

    Jump,      // 27. (relative jump) -> []
    JumpTrue,  // 28. (relative jump) [a] -> []
    JumpFalse, // 29. (relative jump) [a] -> []

    Bind,     // 30. (K = no. binds) [a0, a1, ... aK] {} -> [] {a0, a1, ... aK}
    Unbind,   // 31. (K = no. binds) {a0, a1, ... aK} -> {}
    PushBind, // 32. (id) {aID} [] -> {aID} [aID]
    PushVar,  // 33. (id) [] -> [a]
    MakeVar,  // 34. (id) [a] -> []
}

struct Evaluator {
    program: Program,

    current_function: usize,
    ip: usize,

    data_stack: Vec<i64>,
    return_stack: Vec<usize>,
    bind_stack: Vec<i64>,
    variables: Vec<i64>,
}

impl Evaluator {
    fn new(program: Program) -> Self {
        let variable_size = program.variable_size;
        Self {
            program,
            current_function: 0,
            ip: 0,
            data_stack: Vec::new(),
            return_stack: Vec::new(),
            bind_stack: Vec::new(),
            variables: vec![0; variable_size],
        }
    }

    fn prepare_for_program_evaluation(&mut self) {
        self.current_function = self.program.entry_index;
        self.ip = 0;
    }
}

// pub fn constant_evaluate(code: parser::IRChunk) -> Result<parser::Constant, String> {
// 	let mut stack = Vec::new();
// 	let mut block_stack = Vec::new();

// 	let mut ip = 0;
// 	while ip < code.len() {
// 		let instruction = &code[ip];

// 		use parser::IRKind::*;
// 		match &instruction.kind {
// 			// Literals
// 			PushBool(value) => stack.push(parser::Constant::Bool(*value)),
// 			PushInt(value) => stack.push(parser::Constant::Int(*value)),
// 			PushStr(value) => stack.push(parser::Constant::Str(value.clone())),

// 			// Keywords
// 			End => todo!(),
// 			If => todo!(),
// 			Elif => todo!(),
// 			Else => todo!(),
// 			While => block_stack.push(Block::While(ip)),
// 			Let => todo!(),
// 			Then => todo!(),
// 			Do => todo!(),
// 			In => todo!(),
// 			Def(name) => todo!(),
// 			FunctionArgument(ty) => todo!(),
// 			Var(name) => todo!(),
// 			Struct(name) => todo!(),
// 			StructField(ty) => todo!(),
// 			Include(path) => todo!(),
// 			DashDash => todo!(),

// 			// Operators
// 			Dup => match stack.last().ok_or("Stack underflow!".to_string())? {
// 				parser::Constant::Bool(value) => stack.push(parser::Constant::Bool(*value)),
// 				parser::Constant::Int(value) => stack.push(parser::Constant::Int(*value)),
// 				parser::Constant::Str(value) => stack.push(parser::Constant::Str(value.clone())),
// 			},
// 			Over => {
// 				if stack.len() < 2 {
// 					return Err("Stack underflow!".to_string());
// 				}

// 				match &stack[stack.len() - 2] {
// 					parser::Constant::Bool(value) => stack.push(parser::Constant::Bool(*value)),
// 					parser::Constant::Int(value) => stack.push(parser::Constant::Int(*value)),
// 					parser::Constant::Str(value) => stack.push(parser::Constant::Str(value.clone())),
// 				}
// 			}
// 			Drop => {
// 				stack.pop().ok_or("Stack underflow!".to_string())?;
// 			}
// 			Print => {
// 				let value = stack.pop().ok_or("Stack underflow!".to_string())?;
// 				match value {
// 					parser::Constant::Bool(value) => println!("{}", value),
// 					parser::Constant::Int(value) => println!("{}", value),
// 					parser::Constant::Str(value) => println!("{}", value),
// 				}
// 			}
// 			Add => {
// 				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
// 				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

// 				let a = if let parser::Constant::Int(x) = a {
// 					x
// 				} else {
// 					return Err("Addition is only an integer operation!".to_string());
// 				};

// 				let b = if let parser::Constant::Int(x) = b {
// 					x
// 				} else {
// 					return Err("Addition is only an integer operation!".to_string());
// 				};

// 				stack.push(parser::Constant::Int(a + b));
// 			}
// 			Subtract => {
// 				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
// 				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

// 				let a = if let parser::Constant::Int(x) = a {
// 					x
// 				} else {
// 					return Err("Subtraction is only an integer operation!".to_string());
// 				};

// 				let b = if let parser::Constant::Int(x) = b {
// 					x
// 				} else {
// 					return Err("Subtraction is only an integer operation!".to_string());
// 				};

// 				stack.push(parser::Constant::Int(a - b));
// 			}
// 			Multiply => {
// 				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
// 				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

// 				let a = if let parser::Constant::Int(x) = a {
// 					x
// 				} else {
// 					return Err("Multiplication is only an integer operation!".to_string());
// 				};

// 				let b = if let parser::Constant::Int(x) = b {
// 					x
// 				} else {
// 					return Err("Multiplication is only an integer operation!".to_string());
// 				};

// 				stack.push(parser::Constant::Int(a * b));
// 			}
// 			Divide => {
// 				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
// 				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

// 				let a = if let parser::Constant::Int(x) = a {
// 					x
// 				} else {
// 					return Err("Division is only an integer operation!".to_string());
// 				};

// 				let b = if let parser::Constant::Int(x) = b {
// 					x
// 				} else {
// 					return Err("Division is only an integer operation!".to_string());
// 				};

// 				stack.push(parser::Constant::Int(a / b));
// 			}
// 			Eq => {
// 				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
// 				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

// 				match a {
// 					parser::Constant::Bool(a) => {
// 						if let parser::Constant::Bool(b) = b {
// 							stack.push(parser::Constant::Bool(a == b));
// 						} else {
// 							return Err(
// 								"Cannot evaluate equality between values of different types!".to_string(),
// 							);
// 						}
// 					}
// 					parser::Constant::Int(a) => {
// 						if let parser::Constant::Int(b) = b {
// 							stack.push(parser::Constant::Bool(a == b));
// 						} else {
// 							return Err(
// 								"Cannot evaluate equality between values of different types!".to_string(),
// 							);
// 						}
// 					}
// 					parser::Constant::Str(a) => {
// 						if let parser::Constant::Str(b) = b {
// 							stack.push(parser::Constant::Bool(a == b));
// 						} else {
// 							return Err(
// 								"Cannot evaluate equality between values of different types!".to_string(),
// 							);
// 						}
// 					}
// 				}
// 			}
// 			Call(name) => todo!(),
// 			_ => todo!(),
// 		}
// 	}

// 	if stack.len() > 1 {
// 		return Err("Unhandled data in constant evaluation!".to_string());
// 	}

// 	stack
// 		.pop()
// 		.ok_or("Code does not evaluate to any value!".to_string())
// }

pub fn evaluate(program: Program) -> Result<(), String> {
    let mut evaluator = Evaluator::new(program);
    evaluator.evaluate_global_function()?;
    evaluator.prepare_for_program_evaluation();

    while evaluator.ip
        < evaluator.program.functions[evaluator.current_function]
            .code
            .len()
    {
        let returning_main = evaluator.evaluate_instruction()?;
        if returning_main {
            break;
        }
    }

    Ok(())
}

impl Evaluator {
    fn evaluate_global_function(&mut self) -> Result<(), String> {
        while self.ip < self.program.functions[0].code.len() {
            self.evaluate_instruction()?;
        }
        Ok(())
    }

    fn evaluate_instruction(&mut self) -> Result<bool, String> {
        let instruction = self.program.functions[self.current_function].code[self.ip] as u8;
        self.ip += 1;

        let instruction = unsafe { std::mem::transmute::<u8, Instruction>(instruction) };

        use Instruction::*;
        match instruction {
            _NoOp => panic!("Hit a no-op during evaluation!"),

            PushBool => {
                let value: i64 = unsafe {
                    std::mem::transmute(self.program.functions[self.current_function].code[self.ip])
                };
                self.ip += 1;

                self.data_stack.push(value);
            }
            PushInt => {
                let value: i64 = unsafe {
                    std::mem::transmute(self.program.functions[self.current_function].code[self.ip])
                };
                self.ip += 1;

                self.data_stack.push(value);
            }
            PushStr => {
                let idx = self.program.functions[self.current_function].code[self.ip] as usize;
                self.ip += 1;

                let string = self.program.strings[idx].as_ref().as_ptr();
                self.data_stack.push(string as i64);
            }
            Dup => {
                let top = *self
                    .data_stack
                    .last()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push(top);
            }
            Over => {
                if self.data_stack.len() < 2 {
                    return Err("Stack underflow!".to_string());
                }

                let over = self.data_stack[self.data_stack.len() - 2];
                self.data_stack.push(over);
            }
            Drop => {
                self.data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
            }
            Swap => {
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push(a);
                self.data_stack.push(b);
            }
            PrintBool => {
                let top = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?
                    != 0;
                println!("{}", top);
            }
            PrintInt => {
                let top = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                println!("{}", top);
            }
            PrintStr => {
                let ptr = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())? as *const u8;
                let string = unsafe {
                    string::ptr_to_str(ptr)
                        .map_err(|err| format!("Failed to read string from data stack: {err}"))?
                };
                println!("{}", string);
            }
            Call => {
                let callee_id =
                    self.program.functions[self.current_function].code[self.ip] as usize;

                self.return_stack.push(self.ip + 1);
                self.return_stack.push(self.current_function);

                self.current_function = callee_id;
                self.ip = 0;
            }
            Return => {
                if self.return_stack.len() < 2 {
                    // returning from
                    return Ok(true);
                }

                self.current_function = self
                    .return_stack
                    .pop()
                    .expect("We just checked its length!");
                self.ip = self
                    .return_stack
                    .pop()
                    .expect("We just checked its length!");
            }
            And => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?
                    != 0;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?
                    != 0;
                self.data_stack.push((a && b) as i64);
            }
            Or => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?
                    != 0;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?
                    != 0;
                self.data_stack.push((a || b) as i64);
            }
            Not => {
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?
                    != 0;
                self.data_stack.push((!a) as i64);
            }
            Add => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push(a + b);
            }
            Subtract => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push(a - b);
            }
            Multiply => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push(a * b);
            }
            Divide => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push(a / b);
            }
            Eq => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push((a == b) as i64);
            }
            Neq => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push((a != b) as i64);
            }
            Lt => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push((a < b) as i64);
            }
            Gt => {
                let b = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                let a = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.data_stack.push((a > b) as i64);
            }
            Assign => {
                let ptr = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())? as *mut i64;
                let value = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                unsafe {
                    *ptr = value;
                }
            }
            Load => {
                let ptr = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())? as *const i64;
                self.data_stack.push(unsafe { *ptr });
            }
            LoadStr => todo!(),
            Jump => {
                let jump = self.program.functions[self.current_function].code[self.ip] as i64;
                self.ip += 1;
                self.ip = ((self.ip as i64) + jump) as usize;
            }
            JumpTrue => {
                let jump = self.program.functions[self.current_function].code[self.ip] as i64;
                self.ip += 1;

                let should_jump = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?
                    != 0;
                if should_jump {
                    self.ip = ((self.ip as i64) + jump) as usize;
                }
            }
            JumpFalse => {
                let jump = self.program.functions[self.current_function].code[self.ip] as i64;
                self.ip += 1;

                let should_jump = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?
                    == 0;
                if should_jump {
                    self.ip = ((self.ip as i64) + jump) as usize;
                }
            }
            Bind => {
                let nbinds = self.program.functions[self.current_function].code[self.ip] as usize;
                self.ip += 1;

                let bind_idx = self.data_stack.len() - nbinds;
                let drain = self.data_stack.drain(bind_idx..);
                self.bind_stack.extend(drain);
            }
            Unbind => {
                let nbinds = self.program.functions[self.current_function].code[self.ip] as usize;
                self.ip += 1;

                self.bind_stack.truncate(self.bind_stack.len() - nbinds);
            }
            PushBind => {
                let id = self.program.functions[self.current_function].code[self.ip] as usize;
                self.ip += 1;

                let value = self.bind_stack[id];
                self.data_stack.push(value);
            }
            PushVar => {
                let index = self.program.functions[self.current_function].code[self.ip] as usize;
                self.ip += 1;

                let value = (&self.variables[index]) as *const i64;
                self.data_stack.push(value as i64);
            }
            MakeVar => {
                let index = self.program.functions[self.current_function].code[self.ip] as usize;
                self.ip += 1;

                let value = self
                    .data_stack
                    .pop()
                    .ok_or("Stack underflow!".to_string())?;
                self.variables[index] = value;
            }
            _ => panic!("Invalid instruction: {:?}", instruction),
        }

        Ok(false)
    }
}
