use crate::parser;
use std::collections::HashMap;

type IRIter = <parser::IRChunk as IntoIterator>::IntoIter;

pub fn typecheck(ir_chunks: parser::IRChunks) -> Result<TypedChunks, String> {
	let mut typer = Typer::new();

	let mut typechecked = Vec::new();

	for chunk in ir_chunks {
		let mut ir = chunk.into_iter();
		let typed = typer.typecheck_chunk(&mut ir)?;
		if !typed.is_empty() {
			typechecked.push(typed);
		}
	}
	
	println!("{:#?}", typechecked);

	Ok(typechecked)
}

struct Typer {
	structs: HashMap<String, StructType>,
	functions: HashMap<String, FunctionType>,
	variables: HashMap<String, VariableInfo>,
	next_variable_index: usize,

	// @NOTE:
	// This might be better for this to be a Vec<LinkedList<TypeSignature>>
	// because of the better memory efficiency for branching expressions like `if` and `while`
	//
	type_stacks: Vec<Vec<parser::TypeSignature>>,
	bind_stack: Vec<parser::TypeSignature>,
}

impl Typer {
	fn new() -> Self {
		Self {
			structs: HashMap::new(),
			functions: HashMap::new(),
			variables: HashMap::new(),
			next_variable_index: 0,
			type_stacks: Vec::new(),
			bind_stack: Vec::new(),
		}
	}

	fn type_stack(&mut self) -> &mut Vec<parser::TypeSignature> {
		self
			.type_stacks
			.last_mut()
			.expect("We should have a type stack already")
	}

	fn add_variable(&mut self, name: String, ty: parser::TypeSignature) -> usize {
		self.variables.insert(name, VariableInfo::new(ty, self.next_variable_index));
		self.next_variable_index += 1;
		self.next_variable_index - 1
	}
}

impl Typer {
	fn typecheck_chunk(&mut self, ir: &mut IRIter) -> Result<TypedChunk, String> {
		let mut generated = TypedChunk::new();

		while let Some(i) = ir.next() {
			use parser::IRKind::*;
			match i.kind {
				Def(name) => self.typecheck_function(&mut generated, name, ir)?,
				Var(name) => self.typecheck_variable(&mut generated, name, ir)?,
				Struct(name) => self.typecheck_struct(name, ir)?,
				_ => unreachable!(),
			}
		}

		Ok(generated)
	}

	fn typecheck_expression(
		&mut self,
		generated: &mut TypedChunk,
		ir: parser::IRKind,
		rest: &mut IRIter,
	) -> Result<(), String> {
		use parser::IRKind::*;
		match ir {
			// Literals
			PushBool(value) => {
				generated.push(TypedIR {
					kind: TypedIRKind::PushBool(value),
				});
				self.type_stack().push(parser::TypeSignature::Bool);
			}
			PushInt(value) => {
				generated.push(TypedIR {
					kind: TypedIRKind::PushInt(value),
				});
				self.type_stack().push(parser::TypeSignature::Int);
			}
			PushStr(value) => {
				generated.push(TypedIR {
                    kind: TypedIRKind::PushStr(value),
				});
				self.type_stack().push(parser::TypeSignature::Str);
			}

			// Keywords
			End => return Err("Unexpected `end`!".to_string()),
			If => self.typecheck_if(generated, rest)?,
			Elif => return Err("Unexpected `elif`!".to_string()),
			Else => return Err("Unexpected `else`!".to_string()),
			While => self.typecheck_while(generated, rest)?,
			Then => return Err("Unexpected `then`!".to_string()),
			Do => return Err("Unexpected `do`!".to_string()),
			Def(name) => self.typecheck_function(generated, name, rest)?,
			FunctionArgument(_) => unreachable!(),
			Var(name) => self.typecheck_variable(generated, name, rest)?,
			Struct(name) => self.typecheck_struct(name, rest)?,
			StructField(_) => unreachable!(),
			Include(_) => unreachable!(), // This'll eventually be handled in the parser
			DashDash => unreachable!(),

			// Operators
			Dup => {
				let top = (self
					.type_stack()
					.last()
					.ok_or("Cannot `dup` nonexistant data!".to_string())?)
				.clone();
				self.type_stack().push(top);
				generated.push(TypedIR {
					kind: TypedIRKind::Dup,
				});
			}
			Over => {
				if self.type_stack().len() < 2 {
					return Err(format!(
						"`over` expects at least 2 items on the stack but there were {}!",
						self.type_stack().len()
					));
				}

				let type_stack_len = self.type_stack().len();
				let top = self.type_stack()[type_stack_len - 2].clone();
				self.type_stack().push(top);

				generated.push(TypedIR {
					kind: TypedIRKind::Over,
				});
			}
			Drop => {
				self
					.type_stack()
					.pop()
					.ok_or("Cannot `drop` nonexistant data!".to_string())?;
				generated.push(TypedIR {
					kind: TypedIRKind::Drop,
				});
			}
			Swap => {
				let a = self
					.type_stack()
					.pop()
					.ok_or("Cannot `swap` nonexistant data!".to_string())?;
				let b = self
					.type_stack()
					.pop()
					.ok_or("Cannot `swap` nonexistant data!".to_string())?;
				self.type_stack().push(a);
				self.type_stack().push(b);
				generated.push(TypedIR {
					kind: TypedIRKind::Swap,
				});
			}
			Print => {
				let top = self
					.type_stack()
					.pop()
					.ok_or("Cannot `print` nonexistant data!".to_string())?;
				use parser::TypeSignature::*;
				match top {
					Bool => generated.push(TypedIR {
						kind: TypedIRKind::PrintBool,
					}),
					Int => generated.push(TypedIR {
						kind: TypedIRKind::PrintInt,
					}),
					Str =>
						generated.push(TypedIR {
						kind: TypedIRKind::PrintStr,
					}),
					Ptr(_) => generated.push(TypedIR {
						kind: TypedIRKind::PrintPtr,
					}),
					Struct(_) => unreachable!(),
				}
			}
			And => {
				let b = self.type_stack().pop().ok_or("Cannot `and` nonexistant data!".to_string())?;
				let a = self.type_stack().pop().ok_or("Cannot `and` nonexistant data!".to_string())?;

				if a != parser::TypeSignature::Bool {
					return Err(format!("Cannot `and` something of type `{}`!", a));
				}
				if b != parser::TypeSignature::Bool {
					return Err(format!("Cannot `and` something of type `{}`!", b));
				}

				self.type_stack().push(parser::TypeSignature::Bool);

				generated.push(TypedIR {
					kind: TypedIRKind::And,
				});
			}
			Or => {
				let b = self.type_stack().pop().ok_or("Cannot `or` nonexistant data!".to_string())?;
				let a = self.type_stack().pop().ok_or("Cannot `or` nonexistant data!".to_string())?;

				if a != parser::TypeSignature::Bool {
					return Err(format!("Cannot `or` something of type `{}`!", a));
				}
				if b != parser::TypeSignature::Bool {
					return Err(format!("Cannot `or` something of type `{}`!", b));
				}

				self.type_stack().push(parser::TypeSignature::Bool);

				generated.push(TypedIR {
					kind: TypedIRKind::Or,
				});
			}
			Not => {
				let a = self.type_stack().pop().ok_or("Cannot `or` nonexistant data!".to_string())?;

				if a != parser::TypeSignature::Bool {
					return Err(format!("Cannot `or` something of type `{}`!", a));
				}

				self.type_stack().push(parser::TypeSignature::Bool);

				generated.push(TypedIR {
					kind: TypedIRKind::Not,
				});
			}
			Add => {
				let b = self
					.type_stack()
					.pop()
					.ok_or("Cannot add nonexistant data!".to_string())?;
				let a = self
					.type_stack()
					.pop()
					.ok_or("Cannot add nonexistant data!".to_string())?;

				if a != parser::TypeSignature::Int {
					return Err(format!("Cannot add something of type `{}`!", a));
				}
				if b != parser::TypeSignature::Int {
					return Err(format!("Cannot add something of type `{}`!", b));
				}

				self.type_stack().push(parser::TypeSignature::Int);

				generated.push(TypedIR {
					kind: TypedIRKind::Add,
				});
			}
			Subtract => {
				let b = self
					.type_stack()
					.pop()
					.ok_or("Cannot subtract nonexistant data!".to_string())?;
				let a = self
					.type_stack()
					.pop()
					.ok_or("Cannot subtract nonexistant data!".to_string())?;

				if a != parser::TypeSignature::Int {
					return Err(format!("Cannot subtract something of type `{}`!", a));
				}
				if b != parser::TypeSignature::Int {
					return Err(format!("Cannot subtract something of type `{}`!", b));
				}

				self.type_stack().push(parser::TypeSignature::Int);

				generated.push(TypedIR {
					kind: TypedIRKind::Subtract,
				});
			}
			Multiply => {
				let b = self
					.type_stack()
					.pop()
					.ok_or("Cannot multiply nonexistant data!".to_string())?;
				let a = self
					.type_stack()
					.pop()
					.ok_or("Cannot multiply nonexistant data!".to_string())?;

				if a != parser::TypeSignature::Int {
					return Err(format!("Cannot multiply something of type `{}`!", a));
				}
				if b != parser::TypeSignature::Int {
					return Err(format!("Cannot multiply something of type `{}`!", b));
				}

				self.type_stack().push(parser::TypeSignature::Int);

				generated.push(TypedIR {
					kind: TypedIRKind::Multiply,
				});
			}
			Divide => {
				let b = self
					.type_stack()
					.pop()
					.ok_or("Cannot divide nonexistant data!".to_string())?;
				let a = self
					.type_stack()
					.pop()
					.ok_or("Cannot divide nonexistant data!".to_string())?;

				if a != parser::TypeSignature::Int {
					return Err(format!("Cannot divide something of type `{}`!", a));
				}
				if b != parser::TypeSignature::Int {
					return Err(format!("Cannot divide something of type `{}`!", b));
				}

				self.type_stack().push(parser::TypeSignature::Int);

				generated.push(TypedIR {
					kind: TypedIRKind::Divide,
				});
			}
			Eq => {
				let b = self
					.type_stack()
					.pop()
					.ok_or("Cannot check nonexistant data for equality!".to_string())?;
				let a = self
					.type_stack()
					.pop()
					.ok_or("Cannot check nonexistant data for equality!".to_string())?;

				if a != b {
					return Err(format!(
						"Operands of equality operation have different types! `{}` vs. `{}`!",
						a, b
					));
				}

				self.type_stack().push(parser::TypeSignature::Bool);

				generated.push(TypedIR {
					kind: TypedIRKind::Eq,
				});
			}
			Neq => {
				let b = self
					.type_stack()
					.pop()
					.ok_or("Cannot check nonexistant data for non-equality!".to_string())?;
				let a = self
					.type_stack()
					.pop()
					.ok_or("Cannot check nonexistant data for non-equality!".to_string())?;

				if a != b {
					return Err(format!(
						"Operands of non-equality operation have different types! `{}` vs. `{}`!",
						a, b
					));
				}

				self.type_stack().push(parser::TypeSignature::Bool);

				generated.push(TypedIR {
					kind: TypedIRKind::Neq,
				});
			}
			Lt => {
				let b = self
					.type_stack()
					.pop()
					.ok_or("Cannot compare nonexistant data!".to_string())?;
				let a = self
					.type_stack()
					.pop()
					.ok_or("Cannot compare nonexistant data!".to_string())?;

				if a != parser::TypeSignature::Int {
					return Err(format!("Cannot compare something of type `{}`!", a));
				}
				if b != parser::TypeSignature::Int {
					return Err(format!("Cannot compare something of type `{}`!", b));
				}

				self.type_stack().push(parser::TypeSignature::Bool);

				generated.push(TypedIR {
					kind: TypedIRKind::Lt,
				});
			}
			Gt => {
				let b = self
					.type_stack()
					.pop()
					.ok_or("Cannot compare nonexistant data!".to_string())?;
				let a = self
					.type_stack()
					.pop()
					.ok_or("Cannot compare nonexistant data!".to_string())?;

				if a != parser::TypeSignature::Int {
					return Err(format!("Cannot compare something of type `{}`!", a));
				}
				if b != parser::TypeSignature::Int {
					return Err(format!("Cannot compare something of type `{}`!", b));
				}

				self.type_stack().push(parser::TypeSignature::Bool);

				generated.push(TypedIR {
					kind: TypedIRKind::Gt,
				});
			}
			Assign => {
				// @TODO:
				// handle strings
				//
				let b = self.type_stack().pop().ok_or("Cannot assign nonexistant data to a variable!".to_string())?;
				let a = self.type_stack().pop().ok_or("Cannot assign to nonexistant data!".to_string())?;

				if let parser::TypeSignature::Ptr(ptr_to) = b {
					if a != *ptr_to {
						return Err(format!("Cannot assign to mismatched types! Expected `{}` but found `{}`", ptr_to, a));
					}
				} else {
					return Err(format!("Cannot assign to something of non-pointer type! Found `{}`!", b));
				}

				generated.push(TypedIR {
					kind: TypedIRKind::Assign,
				});
			}
			Load => {
				let a = self.type_stack().pop().ok_or("Cannot load non-existant data!".to_string())?;
				match a {
					parser::TypeSignature::Ptr(ptr_to) => {
						if let parser::TypeSignature::Str = *ptr_to {
							self.type_stack().push(parser::TypeSignature::Str);
							generated.push(TypedIR {
								kind: TypedIRKind::LoadStr,
							});
						} else {
							self.type_stack().push(*ptr_to);
							generated.push(TypedIR {
								kind: TypedIRKind::Load,
							});
						}
					}
					_ => return Err(format!("Cannot load something of type `{}`!", a)),
				}
			}
			Call(name) => {
				let function_type = self
					.functions
					.get(&name)
					.expect("Unresolved identifiers should be caught during parsing");

				if !self
					.type_stacks
					.last()
					.expect("We should have a type stack")
					.ends_with(function_type.parameters.as_slice())
				{
					return Err(format!(
						"Incorrect types for call to `{}`! Stack: {}. Parameters: {}",
						name, 
						parser::DisplayVec(self.type_stacks.last().expect("We should have a type stack")), 
						parser::DisplayVec(&function_type.parameters)
					));
				}

				let type_stack_len = self
					.type_stacks
					.last()
					.expect("We should have a type stack")
					.len();
				self
					.type_stacks
					.last_mut()
					.expect("We should have a type stack")
					.truncate(type_stack_len - function_type.parameters.len());
				self
					.type_stacks
					.last_mut()
					.expect("We should have a type stack")
					.extend_from_slice(function_type.returns.as_slice());

				generated.push(TypedIR {
					kind: TypedIRKind::Call(name),
				});
			}
			Bind(nbinds) => {
				let split_idx = self.type_stack().len() - nbinds;
				let drain = self.type_stacks.last_mut().expect("We should have a type stack").drain(split_idx..);
				self.bind_stack.extend(drain);

				generated.push(TypedIR {
					kind: TypedIRKind::Bind(nbinds),
				});
			}
			Unbind(nbinds) => {
				self.bind_stack.truncate(self.bind_stack.len() - nbinds);
				generated.push(TypedIR {
					kind: TypedIRKind::Unbind(nbinds),
				});
			}
			PushBind(id) => {
				let ty = self.bind_stack[id].clone();
				self.type_stack().push(ty);
				generated.push(TypedIR { kind: TypedIRKind::PushBind(id) });
			}
			PushVar(name) => {
				let var = self.variables.get(&name).expect("Unknown identifiers should be handled during parsing");
				self
				.type_stacks
				.last_mut()
				.expect("We should have a type stack")
				.push(parser::TypeSignature::Ptr(
					Box::new(var.ty.clone()))
				);
				generated.push(TypedIR { kind: TypedIRKind::PushVar(var.index) });
			}
		}
		Ok(())
	}

	fn typecheck_function(
		&mut self,
		generated: &mut TypedChunk,
		name: String,
		ir: &mut IRIter,
	) -> Result<(), String> {
		generated.push(TypedIR {
			kind: TypedIRKind::Def(name.clone()),
		});

		let mut function_type = FunctionType::new();

		// parse parameter and return types for function
		{
			let mut parsing_return_types = false;
			while let Some(i) = ir.next() {
				use parser::IRKind::*;
				match i.kind {
					Do => break,
					FunctionArgument(type_signature) => {
						let types = if parsing_return_types {
							&mut function_type.returns
						} else {
							&mut function_type.parameters
						};

						match type_signature {
							parser::TypeSignature::Struct(name) => {
								let struct_type = self
									.structs
									.get(&name)
									.expect("Unresolved identifiers should be caught during parsing");
								types.extend_from_slice(struct_type.field_types.as_slice());
							}
							_ => types.push(type_signature),
						}
					}
					DashDash => parsing_return_types = true,
					_ => unreachable!(),
				}
			}
		}

		self.type_stacks.push(function_type.parameters.clone());
		self.functions.insert(name.clone(), function_type);

		while let Some(i) = ir.next() {
			use parser::IRKind::*;
			match i.kind {
				End => break,
				_ => self.typecheck_expression(generated, i.kind, ir)?,
			}
		}

		if *self
			.type_stacks
			.last()
			.expect("We should have a type stack")
			!= self
				.functions
				.get(&name)
				.expect("We inserted it before checking the body")
				.returns
		{
			return Err(format!(
				"The function `{}` doesn't match its return types! Expected: {} vs. Actual {}",
				name,
				parser::DisplayVec(&self.functions.get(&name).expect("We inserted it before checking the body").returns),
				parser::DisplayVec(self.type_stacks.last().expect("We should have a type stack")),
			));
		}

		self
			.type_stacks
			.pop()
			.expect("We pushed one before typechecking the body so it should be here");

		Ok(())
	}

	fn typecheck_variable(&mut self, generated: &mut TypedChunk, name: String, ir: &mut IRIter) -> Result<(), String> {
		self.type_stacks.push(Vec::new());

		generated.push(TypedIR { kind: TypedIRKind::Var });

		while let Some(i) = ir.next() {
			use parser::IRKind::*;
			match i.kind {
				End => break,
				_ => self.typecheck_expression(generated, i.kind, ir)?,
			}
		}

		if self.type_stack().len() != 1 {
			return Err("Body of `var` expression does not evaluate to a single value!".to_string());
		}

		let var_type = self.type_stack().pop().expect("We just checked its length");
		let var_index = self.add_variable(name, var_type);

		generated.push(TypedIR { kind: TypedIRKind::MakeVar(var_index) });

		self.type_stacks.pop().expect("We push a new stack for the var");

		Ok(())
	}

	fn typecheck_if(&mut self, generated: &mut TypedChunk, ir: &mut IRIter) -> Result<(), String> {
		let type_stack_before_if = self.type_stack().clone();
		let mut type_stack_before_branch = None::<Vec<parser::TypeSignature>>;

		generated.push(TypedIR {
			kind: TypedIRKind::If,
		});

		while let Some(i) = ir.next() {
			use parser::IRKind::*;
			match i.kind {
				Then => {
					let top = self
						.type_stack()
						.pop()
						.ok_or("No value on stack for condition of `if` expression!".to_string())?;
					if top != parser::TypeSignature::Bool {
						return Err(format!(
							"Type on stack for condition of `if` expression should be {} but found {}",
							parser::TypeSignature::Bool,
							top,
						));
					}
					generated.push(TypedIR {
						kind: TypedIRKind::Then,
					});
				}
				Elif => {
					if let Some(type_stack_before_branch) = &type_stack_before_branch {
						if self.type_stack() != type_stack_before_branch {
							return Err(format!(
								"A branch of `if` expression returns different types to other branches! Expected: {} vs. Actual: {}", 
								parser::DisplayVec(type_stack_before_branch), 
								parser::DisplayVec(self.type_stack()),
							));
						}
					} else {
						type_stack_before_branch = Some(self.type_stack().clone());
					}

					generated.push(TypedIR {
						kind: TypedIRKind::Elif,
					});
				}
				Else => {
					if let Some(type_stack_before_branch) = &type_stack_before_branch {
						if self.type_stack() != type_stack_before_branch {
							return Err(format!(
								"A branch of `if` expression returns different types to other branches! Expected: {} vs. Actual: {}", 
								parser::DisplayVec(type_stack_before_branch),
								parser::DisplayVec(self.type_stack()),
							));
						}
					} else {
						type_stack_before_branch = Some(self.type_stack().clone());
					}

					generated.push(TypedIR {
						kind: TypedIRKind::Else,
					});
				}
				End => {
					if let Some(type_stack_before_branch) = &type_stack_before_branch {
						if self.type_stack() != type_stack_before_branch {
							return Err(format!(
								"A branch of `if` expression returns different types to other branches! Expected: {} vs. Actual: {}", 
								parser::DisplayVec(type_stack_before_branch),
								parser::DisplayVec(self.type_stack()),
							));
						}
					} else {
						if *self.type_stack() != type_stack_before_if {
							return Err(format!(
								"`if` expression ends with altered type stack! Before: {} vs. After: {}",
								parser::DisplayVec(&type_stack_before_if),
								parser::DisplayVec(self.type_stack()),
							));
						}
					}

					generated.push(TypedIR {
						kind: TypedIRKind::End,
					});

					break;
				}
				_ => self.typecheck_expression(generated, i.kind, ir)?,
			}
		}

		Ok(())
	}

	fn typecheck_while(&mut self, generated: &mut TypedChunk, ir: &mut IRIter) -> Result<(), String> {
		let type_stack_before_loop = self.type_stack().clone();

		generated.push(TypedIR {
			kind: TypedIRKind::While,
		});

		while let Some(i) = ir.next() {
			use parser::IRKind::*;
			match i.kind {
				End => {
					generated.push(TypedIR {
						kind: TypedIRKind::End,
					});
					break;
				}
				Do => {
					let condition = self
						.type_stack()
						.pop()
						.ok_or("`while` loop requires a condition but no data is present!".to_string())?;
					if condition != parser::TypeSignature::Bool {
						return Err(format!(
							"`while` loop requires its condition value to be `bool` but found {}",
							condition
						));
					}

					generated.push(TypedIR {
						kind: TypedIRKind::Do,
					});
				}
				_ => self.typecheck_expression(generated, i.kind, ir)?,
			}
		}

		if type_stack_before_loop != *self.type_stack() {
			return Err(format!(
				"`while` loop ends with altered type stack! Expected: {} vs. Actual: {}", 
				parser::DisplayVec(&type_stack_before_loop),
				parser::DisplayVec(self.type_stack()),
			));
		}

		Ok(())
	}

	fn typecheck_struct(&mut self, name: String, ir: &mut IRIter) -> Result<(), String> {
		let mut struct_type = StructType::new();

		while let Some(i) = ir.next() {
			use parser::IRKind::*;
			match i.kind {
				End => break,
				StructField(ty) => {
					struct_type.field_types.push(ty);
				}
				_ => unreachable!(),
			}
		}

		self.structs.insert(name, struct_type);

		Ok(())
	}
}

struct StructType {
	field_types: Vec<parser::TypeSignature>,
}

impl StructType {
	fn new() -> Self {
		Self {
			field_types: Vec::new(),
		}
	}
}

struct FunctionType {
	parameters: Vec<parser::TypeSignature>,
	returns: Vec<parser::TypeSignature>,
}

impl FunctionType {
	fn new() -> Self {
		Self {
			parameters: Vec::new(),
			returns: Vec::new(),
		}
	}
}

struct VariableInfo {
	ty: parser::TypeSignature,
	index: usize,
}

impl VariableInfo {
	fn new(ty: parser::TypeSignature, index: usize) -> Self {
		Self { ty, index }
	}
}

#[derive(Debug)]
pub struct TypedIR {
	pub kind: TypedIRKind,
}

#[derive(Debug)]
pub enum TypedIRKind {
	// Literals
	PushBool(bool),
	PushInt(i64),
	PushStr(String),

	// Keywords
	End,
	If,
	Elif,
	Else,
	While,
	Then,
	Do,
	Def(String),
	Var,

	// Operators
	Dup,
	Over,
	Drop,
	Swap,
	PrintBool,
	PrintInt,
	PrintStr,
	PrintPtr,
	And,
	Or,
	Not,
	Add,
	Subtract,
	Multiply,
	Divide,
	Eq,
	Neq,
	Lt,
	Gt,
	Assign,
	Load,
	LoadStr,
	Call(String),
	Bind(usize),
	Unbind(usize),
	PushBind(usize),
	PushVar(usize),
	MakeVar(usize)
}

pub type TypedChunk = Vec<TypedIR>;
pub type TypedChunks = Vec<TypedChunk>;
