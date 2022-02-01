use crate::evaluator;
use crate::parser;
use std::collections::HashMap;

type IRIter = <parser::IRChunk as IntoIterator>::IntoIter;

pub fn compile(ir_chunks: parser::IRChunks) -> Result<evaluator::Program, String> {
	// @NOTE:
	// We're assumming that we typecheck!
	//

	let mut compiler = Compiler::new();

	for chunk in ir_chunks {
		let mut ir = chunk.into_iter();
		match ir.next().expect("We filter out empty chunks in the parser") {
			parser::IR {
				kind: parser::IRKind::Def(ident),
			} => compiler.compile_function(ident.clone(), &mut ir)?,
			parser::IR {
				kind: parser::IRKind::Var(_),
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
}

impl Compiler {
	fn compile_function(&mut self, name: String, ir: &mut IRIter) -> Result<(), String> {
		self.add_function(name);

		while let Some(i) = ir.next() {
			use parser::IRKind::*;
			match i.kind {
				// Literals
				PushBool(value) => self.emit_push_bool(value),
				PushInt(value) => self.emit_push_int(value),
				PushStr(value) => self.emit_push_str(value),

				// Keywords
				End => break,
				If => self.compile_if(ir)?,
				Elif => unreachable!(),
				Else => unreachable!(),
				While => self.compile_while(ir)?,
				Let => todo!(),
				Then => unreachable!(),
				Do => {} // @HACK: Just to get things working: this is for `def func do ...`
				In => todo!(),
				Def(name) => self.compile_function(name, ir)?,
				FunctionArgument(type_signature) => unreachable!(), // @TODO: Do argument handling before hand
				Var(name) => todo!(),
				Struct(_) => {
					todo!("In the future this'll probably be handling during type checking anyway")
				}
				StructMember(_, _) => unreachable!(),
				Include(_) => unreachable!(), // This'll eventually be handled in the parser
				DashDash => unreachable!(),

				// Operators
				Dup => self.emit_instruction(evaluator::Instruction::Dup),
				Over => self.emit_instruction(evaluator::Instruction::Over),
				Drop => self.emit_instruction(evaluator::Instruction::Drop),
				Print => self.emit_instruction(evaluator::Instruction::PrintInt), // @HACK: For now we only print ints
				Add => self.emit_instruction(evaluator::Instruction::Add),
				Subtract => self.emit_instruction(evaluator::Instruction::Subtract),
				Multiply => self.emit_instruction(evaluator::Instruction::Multiply),
				Divide => self.emit_instruction(evaluator::Instruction::Divide),
				Eq => self.emit_instruction(evaluator::Instruction::Eq),
				Call(name) => {
					let function_id = self
						.get_function_id(&name)
						.expect(format!("No function named `{}` in function map!", name).as_str());
					self.emit_call(function_id);
				}
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
		todo!()
	}

	fn compile_while(&mut self, ir: &mut IRIter) -> Result<(), String> {
		todo!()
	}
}

// @NOTE @TODO:
// We don't need this to be u64 if we can do some bit manipulation
// This should be a Vec<u8> in the future but for now we're using
// u64 to simplify the compiler
//
pub type Code = Vec<u64>;
