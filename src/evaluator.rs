use crate::compiler;
use crate::parser;

#[derive(Debug)]
pub struct Function {
	// pub parameters: Vec<parser::TypeSignature>,
	// pub returns: Vec<parser::TypeSignature>,
	pub code: compiler::Code,
}

impl Function {
	pub fn new() -> Self {
		Self {
			// parameters: Vec::new(),
			// returns: Vec::new(),
			code: compiler::Code::new(),
		}
	}
}

#[derive(Debug)]
pub struct Program {
	pub entry_index: usize,
	pub functions: Vec<Function>,
}

// Key:
// () = arguments in the code
// [] = arguments on the stack
// -a = peek argument (doesn't pop)
//
#[derive(Debug)]
pub enum Instruction {
	NoOp, // 0. Just to reserve 0

	PushBool, // 1. (a) -> [a]
	PushInt,  // 2. (a) -> [a]
	PushStr,  // 3. (size, ptr) -> [size, ptr]

	Dup,  // 4. [-a] -> [a, a]
	Over, // 5. [-a, -b] -> [a, b, a]
	Drop, // 6. [a] -> []
	Swap, // 7. [a, b] -> [b, a]

	PrintBool, // 8. [a] -> []
	PrintInt,  // 9. [a] -> []
	PrintStr,  // 10. [size, ptr] -> []

	Call,   // 11. (fid) -> [return values]
	Return, // 12. [] -> []

	Add,      // 13. [a, b] -> [c]
	Subtract, // 14. [a, b] -> [c]
	Multiply, // 15. [a, b] -> [c]
	Divide,   // 16. [a, b] -> [c]

	Eq,  // 17. [a, b] -> [c]
	Neq, // 18. [a, b] -> [c]
	Lt,  // 19. [a, b] -> [c]
	Gt,  // 20. [a, b] -> [c]

	Jump,      // 21. (relative jump) -> []
	JumpTrue,  // 22. (relative jump) [a] -> []
	JumpFalse, // 23. (relative jump) [a] -> []
}

enum Block {
	If(usize),
	While(usize),
}

pub fn constant_evaluate(code: parser::IRChunk) -> Result<parser::Constant, String> {
	let mut stack = Vec::new();
	let mut block_stack = Vec::new();

	let mut ip = 0;
	while ip < code.len() {
		let instruction = &code[ip];

		use parser::IRKind::*;
		match &instruction.kind {
			// Literals
			PushBool(value) => stack.push(parser::Constant::Bool(*value)),
			PushInt(value) => stack.push(parser::Constant::Int(*value)),
			PushStr(value) => stack.push(parser::Constant::Str(value.clone())),

			// Keywords
			End => todo!(),
			If => todo!(),
			Elif => todo!(),
			Else => todo!(),
			While => block_stack.push(Block::While(ip)),
			Let => todo!(),
			Then => todo!(),
			Do => todo!(),
			In => todo!(),
			Def(name) => todo!(),
			FunctionArgument(ty) => todo!(),
			Var(name) => todo!(),
			Struct(name) => todo!(),
			StructField(ty) => todo!(),
			Include(path) => todo!(),
			DashDash => todo!(),

			// Operators
			Dup => match stack.last().ok_or("Stack underflow!".to_string())? {
				parser::Constant::Bool(value) => stack.push(parser::Constant::Bool(*value)),
				parser::Constant::Int(value) => stack.push(parser::Constant::Int(*value)),
				parser::Constant::Str(value) => stack.push(parser::Constant::Str(value.clone())),
			},
			Over => {
				if stack.len() < 2 {
					return Err("Stack underflow!".to_string());
				}

				match &stack[stack.len() - 2] {
					parser::Constant::Bool(value) => stack.push(parser::Constant::Bool(*value)),
					parser::Constant::Int(value) => stack.push(parser::Constant::Int(*value)),
					parser::Constant::Str(value) => stack.push(parser::Constant::Str(value.clone())),
				}
			}
			Drop => {
				stack.pop().ok_or("Stack underflow!".to_string())?;
			}
			Print => {
				let value = stack.pop().ok_or("Stack underflow!".to_string())?;
				match value {
					parser::Constant::Bool(value) => println!("{}", value),
					parser::Constant::Int(value) => println!("{}", value),
					parser::Constant::Str(value) => println!("{}", value),
				}
			}
			Add => {
				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

				let a = if let parser::Constant::Int(x) = a {
					x
				} else {
					return Err("Addition is only an integer operation!".to_string());
				};

				let b = if let parser::Constant::Int(x) = b {
					x
				} else {
					return Err("Addition is only an integer operation!".to_string());
				};

				stack.push(parser::Constant::Int(a + b));
			}
			Subtract => {
				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

				let a = if let parser::Constant::Int(x) = a {
					x
				} else {
					return Err("Subtraction is only an integer operation!".to_string());
				};

				let b = if let parser::Constant::Int(x) = b {
					x
				} else {
					return Err("Subtraction is only an integer operation!".to_string());
				};

				stack.push(parser::Constant::Int(a - b));
			}
			Multiply => {
				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

				let a = if let parser::Constant::Int(x) = a {
					x
				} else {
					return Err("Multiplication is only an integer operation!".to_string());
				};

				let b = if let parser::Constant::Int(x) = b {
					x
				} else {
					return Err("Multiplication is only an integer operation!".to_string());
				};

				stack.push(parser::Constant::Int(a * b));
			}
			Divide => {
				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

				let a = if let parser::Constant::Int(x) = a {
					x
				} else {
					return Err("Division is only an integer operation!".to_string());
				};

				let b = if let parser::Constant::Int(x) = b {
					x
				} else {
					return Err("Division is only an integer operation!".to_string());
				};

				stack.push(parser::Constant::Int(a / b));
			}
			Eq => {
				let a = stack.pop().ok_or("Stack underflow!".to_string())?;
				let b = stack.pop().ok_or("Stack underflow!".to_string())?;

				match a {
					parser::Constant::Bool(a) => {
						if let parser::Constant::Bool(b) = b {
							stack.push(parser::Constant::Bool(a == b));
						} else {
							return Err(
								"Cannot evaluate equality between values of different types!".to_string(),
							);
						}
					}
					parser::Constant::Int(a) => {
						if let parser::Constant::Int(b) = b {
							stack.push(parser::Constant::Bool(a == b));
						} else {
							return Err(
								"Cannot evaluate equality between values of different types!".to_string(),
							);
						}
					}
					parser::Constant::Str(a) => {
						if let parser::Constant::Str(b) = b {
							stack.push(parser::Constant::Bool(a == b));
						} else {
							return Err(
								"Cannot evaluate equality between values of different types!".to_string(),
							);
						}
					}
				}
			}
			Call(name) => todo!(),
			_ => todo!(),
		}
	}

	if stack.len() > 1 {
		return Err("Unhandled data in constant evaluation!".to_string());
	}

	stack
		.pop()
		.ok_or("Code does not evaluate to any value!".to_string())
}

pub fn evaluate(program: Program) -> Result<(), String> {
	let mut current_function = program.entry_index;
	let mut ip = 0;

	let mut data_stack = Vec::new();
	let mut return_stack: Vec<usize> = Vec::new();

	while ip < program.functions[current_function].code.len() {
		let instruction = program.functions[current_function].code[ip] as u8;
		ip += 1;

		let instruction = unsafe { std::mem::transmute::<u8, Instruction>(instruction) };

		use Instruction::*;
		match instruction {
			NoOp => panic!("Hit a no-op during evaluation!"),

			PushBool => {
				let value: i64 =
					unsafe { std::mem::transmute(program.functions[current_function].code[ip]) };
				ip += 1;

				data_stack.push(value);
			}
			PushInt => {
				let value: i64 =
					unsafe { std::mem::transmute(program.functions[current_function].code[ip]) };
				ip += 1;

				data_stack.push(value);
			}
			PushStr => {
				let size = program.functions[current_function].code[ip];
				ip += 1;

				let ptr: *const u8 =
					unsafe { std::mem::transmute(program.functions[current_function].code[ip]) };
				ip += 1;

				data_stack.push(size as i64);
				data_stack.push(ptr as i64);
			}
			Dup => {
				let top = *data_stack.last().ok_or("Stack underflow!".to_string())?;
				data_stack.push(top);
			}
			Over => {
				if data_stack.len() < 2 {
					return Err("Stack underflow!".to_string());
				}

				let over = data_stack[data_stack.len() - 2];
				data_stack.push(over);
			}
			Drop => {
				data_stack.pop().ok_or("Stack underflow!".to_string())?;
			}
			Swap => {
				let a = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				let b = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				data_stack.push(a);
				data_stack.push(b);
			}
			PrintBool => {
				let top = data_stack.pop().ok_or("Stack underflow!".to_string())? != 0;
				println!("{}", top);
			}
			PrintInt => {
				let top = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				println!("{}", top);
			}
			PrintStr => {
				let ptr = data_stack.pop().ok_or("Stack underflow!".to_string())? as *const u8;
				let size = data_stack.pop().ok_or("Stack underflow!".to_string())? as usize;
				let string = std::str::from_utf8(unsafe { std::slice::from_raw_parts(ptr, size) });
				match string {
					Ok(s) => println!("{}", s),
					Err(err) => return Err(format!("{}", err)),
				}
			}
			Call => {
				let callee_id = program.functions[current_function].code[ip] as usize;

				return_stack.push(ip + 1);
				return_stack.push(current_function);

				current_function = callee_id;
				ip = 0;
			}
			Return => {
				if return_stack.len() < 2 {
					// returning from main
					break;
				}

				current_function = return_stack.pop().expect("We just checked its length!");
				ip = return_stack.pop().expect("We just checked its length!");
			}
			Add => {
				let b = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				let a = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				data_stack.push(a + b);
			}
			Subtract => {
				let b = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				let a = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				data_stack.push(a - b);
			}
			Multiply => {
				let b = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				let a = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				data_stack.push(a * b);
			}
			Divide => {
				let b = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				let a = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				data_stack.push(a / b);
			}
			Eq => {
				let b = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				let a = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				data_stack.push((a == b) as i64);
			}
			Neq => {
				let b = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				let a = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				data_stack.push((a != b) as i64);
			}
			Lt => {
				let b = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				let a = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				data_stack.push((a < b) as i64);
			}
			Gt => {
				let b = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				let a = data_stack.pop().ok_or("Stack underflow!".to_string())?;
				data_stack.push((a > b) as i64);
			}
			Jump => {
				let jump = program.functions[current_function].code[ip] as i64;
				ip += 1;
				ip = ((ip as i64) + jump) as usize;
			}
			JumpTrue => {
				let jump = program.functions[current_function].code[ip] as i64;
				ip += 1;

				let should_jump = data_stack.pop().ok_or("Stack underflow!".to_string())? != 0;
				if should_jump {
					ip = ((ip as i64) + jump) as usize;
				}
			}
			JumpFalse => {
				let jump = program.functions[current_function].code[ip] as i64;
				ip += 1;

				let should_jump = data_stack.pop().ok_or("Stack underflow!".to_string())? == 0;
				if should_jump {
					ip = ((ip as i64) + jump) as usize;
				}
			}
			_ => panic!("Invalid instruction: {:?}", instruction),
		}
	}

	Ok(())
}
