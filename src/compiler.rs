use crate::evaluator;
use crate::typer;
use std::collections::HashMap;

type IRIter = <typer::TypedChunk as IntoIterator>::IntoIter;

pub fn compile(ir_chunks: typer::TypedChunks) -> Result<evaluator::Program, String> {
	// @NOTE:
	// We're assumming that we typecheck!
	//

	let mut compiler = Compiler::new();

	for chunk in ir_chunks {
		let mut ir = chunk.into_iter();
		match ir.next().expect("We filter out empty chunks in the parser") {
			typer::TypedIR {
				kind: typer::TypedIRKind::Def(ident),
			} => compiler.compile_function(ident.clone(), &mut ir)?,
			typer::TypedIR {
				kind: typer::TypedIRKind::Var(_),
			} => todo!("Implement compilation of variables at top-level"),
			_ => unreachable!(),
		}
	}

	Ok(compiler.program)
}

struct Compiler {
	program: evaluator::Program,
	function_map: HashMap<String, usize>,
	function_stack: Vec<usize>,
}

impl Compiler {
	fn new() -> Self {
		Self {
			program: evaluator::Program {
				entry_index: 0,
				functions: Vec::new(),
			},
			function_map: HashMap::new(),
			function_stack: Vec::new(),
		}
	}

	fn current_function_id(&self) -> usize {
		*self
			.function_stack
			.last()
			.expect("We should have at least one function on the stack!")
	}

	fn add_function(&mut self, name: String) {
		let function_id = self.program.functions.len();
		self.program.functions.push(evaluator::Function::new());

		if name == "main" {
			self.program.entry_index = function_id;
		}

		self.function_map.insert(name, function_id);
		self.function_stack.push(function_id);
	}

	fn get_function_id(&self, name: &String) -> Option<usize> {
		self.function_map.get(name).map(|id| *id)
	}
}

impl Compiler {
	fn emit_push_bool(&mut self, value: bool) {
		let current_function_id = self.current_function_id();
		let current_function = &mut self.program.functions[current_function_id];

		current_function
			.code
			.push(evaluator::Instruction::PushBool as u64);

		current_function.code.push(value as u64);
	}

	fn emit_push_int(&mut self, value: i64) {
		let current_function_id = self.current_function_id();
		let current_function = &mut self.program.functions[current_function_id];

		current_function
			.code
			.push(evaluator::Instruction::PushInt as u64);

		current_function
			.code
			.push(unsafe { std::mem::transmute::<i64, u64>(value) });
	}

	fn emit_push_str(&mut self, value: String) {
		todo!()
	}

	fn emit_instruction(&mut self, instruction: evaluator::Instruction) {
		let current_function_id = self.current_function_id();
		let current_function = &mut self.program.functions[current_function_id];
		current_function.code.push(instruction as u64);
	}

	fn emit_call(&mut self, function_id: usize) {
		let current_function_id = self.current_function_id();
		let current_function = &mut self.program.functions[current_function_id];

		current_function
			.code
			.push(evaluator::Instruction::Call as u64);

		current_function.code.push(function_id as u64);
	}

	fn emit_jump(&mut self, jump: i64) {
		let current_function_id = self.current_function_id();
		let current_function = &mut self.program.functions[current_function_id];

		current_function
			.code
			.push(evaluator::Instruction::Jump as u64);

		current_function
			.code
			.push(unsafe { std::mem::transmute::<i64, u64>(jump) });
	}

	fn emit_jump_true(&mut self, jump: i64) {
		let current_function_id = self.current_function_id();
		let current_function = &mut self.program.functions[current_function_id];

		current_function
			.code
			.push(evaluator::Instruction::JumpTrue as u64);

		current_function
			.code
			.push(unsafe { std::mem::transmute::<i64, u64>(jump) });
	}

	fn emit_jump_false(&mut self, jump: i64) {
		let current_function_id = self.current_function_id();
		let current_function = &mut self.program.functions[current_function_id];

		current_function
			.code
			.push(evaluator::Instruction::JumpFalse as u64);

		current_function
			.code
			.push(unsafe { std::mem::transmute::<i64, u64>(jump) });
	}

	fn patch_jump(&mut self, jump_index: usize) {
		let current_function_id = self.current_function_id();
		let current_function = &mut self.program.functions[current_function_id];

		current_function.code[jump_index] = (current_function.code.len() - jump_index - 1) as u64;
	}
}

impl Compiler {
	fn compile_expression(
		&mut self,
		ir: typer::TypedIRKind,
		rest: &mut IRIter,
	) -> Result<(), String> {
		use typer::TypedIRKind::*;
		match ir {
			// Literals
			PushBool(value) => self.emit_push_bool(value),
			PushInt(value) => self.emit_push_int(value),
			PushStr(value) => self.emit_push_str(value),

			// Keywords
			End => return Err("Unexpected `end`!".to_string()),
			If => self.compile_if(rest)?,
			Elif => return Err("Unexpected `elif`!".to_string()),
			Else => return Err("Unexpected `else`!".to_string()),
			While => self.compile_while(rest)?,
			Let => todo!(),
			Then => return Err("Unexpected `then`!".to_string()),
			Do => return Err("Unexpected `do`!".to_string()),
			In => return Err("Unexpected `in`!".to_string()),
			Def(name) => self.compile_function(name, rest)?,
			Var(name) => todo!(),
			// DashDash => unreachable!(),

			// Operators
			Dup => self.emit_instruction(evaluator::Instruction::Dup),
			Over => self.emit_instruction(evaluator::Instruction::Over),
			Drop => self.emit_instruction(evaluator::Instruction::Drop),
			Swap => self.emit_instruction(evaluator::Instruction::Swap),
			// Print => self.emit_instruction(evaluator::Instruction::PrintInt), // @HACK: For now we only print ints
			PrintBool => self.emit_instruction(evaluator::Instruction::PrintBool),
			PrintInt => self.emit_instruction(evaluator::Instruction::PrintInt),
			PrintStr => self.emit_instruction(evaluator::Instruction::PrintStr),
			PrintPtr => todo!(),
			Add => self.emit_instruction(evaluator::Instruction::Add),
			Subtract => self.emit_instruction(evaluator::Instruction::Subtract),
			Multiply => self.emit_instruction(evaluator::Instruction::Multiply),
			Divide => self.emit_instruction(evaluator::Instruction::Divide),
			Eq => self.emit_instruction(evaluator::Instruction::Eq),
			Neq => self.emit_instruction(evaluator::Instruction::Neq),
			Lt => self.emit_instruction(evaluator::Instruction::Lt),
			Gt => self.emit_instruction(evaluator::Instruction::Gt),
			Call(name) => {
				let function_id = self
					.get_function_id(&name)
					.expect(format!("No function named `{}` in function map!", name).as_str());
				self.emit_call(function_id);
			}
		}
		Ok(())
	}

	fn compile_function(&mut self, name: String, ir: &mut IRIter) -> Result<(), String> {
		self.add_function(name);

		while let Some(i) = ir.next() {
			use typer::TypedIRKind::*;
			match i.kind {
				End => break,
				_ => self.compile_expression(i.kind, ir)?,
			}
		}

		self.emit_instruction(evaluator::Instruction::Return);

		self
			.function_stack
			.pop()
			.expect("We should have pushed one on at the start of the function!");

		Ok(())
	}

	fn compile_if(&mut self, ir: &mut IRIter) -> Result<(), String> {
		let mut jump_index = Some(0);
		let mut exits = Vec::new();

		while let Some(i) = ir.next() {
			use typer::TypedIRKind::*;
			match i.kind {
				End => {
					if let Some(jump_index) = jump_index {
						self.patch_jump(jump_index);
					}
					exits.into_iter().for_each(|index| self.patch_jump(index));
					break;
				}
				Elif => {
					self.emit_jump(-1);
					exits.push(
						self.program.functions[self.current_function_id()]
							.code
							.len() - 1,
					);
					self.patch_jump(jump_index.expect("We should have a jump index!"));
				}
				Else => {
					self.emit_jump(-1);
					exits.push(
						self.program.functions[self.current_function_id()]
							.code
							.len() - 1,
					);
					self.patch_jump(jump_index.expect("We should have a jump index!"));
					jump_index = None;
				}
				Then => {
					self.emit_jump_false(-1);
					jump_index = Some(
						self.program.functions[self.current_function_id()]
							.code
							.len() - 1,
					);
				}
				_ => self.compile_expression(i.kind, ir)?,
			}
		}

		Ok(())
	}

	fn compile_while(&mut self, ir: &mut IRIter) -> Result<(), String> {
		let while_index = self.program.functions[self.current_function_id()]
			.code
			.len();
		let mut do_index = 0;

		while let Some(i) = ir.next() {
			use typer::TypedIRKind::*;
			match i.kind {
				End => {
					self.emit_jump(
						(while_index.wrapping_sub(
							self.program.functions[self.current_function_id()]
								.code
								.len() + 2, // plus 2 because of the jump instruction itself
						)) as i64,
					);
					self.patch_jump(do_index);
					break;
				}
				Do => {
					self.emit_jump_false(-1);
					do_index = self.program.functions[self.current_function_id()]
						.code
						.len() - 1;
				}
				_ => self.compile_expression(i.kind, ir)?,
			}
		}

		Ok(())
	}
}

// @NOTE @TODO:
// We don't need this to be u64 if we can do some bit manipulation
// This should be a Vec<u8> in the future but for now we're using
// u64 to simplify the compiler
//
pub type Code = Vec<u64>;
