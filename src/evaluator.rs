use crate::parser;

pub fn constant_evaluate(code: Vec<parser::IR>) -> Result<parser::Constant, String> {
	let mut stack: Vec<parser::Constant> = Vec::new();

	let mut iter = code.into_iter();
	while let Some(instruction) = iter.next() {
		use parser::IRKind::*;
		match instruction.kind {
			// Literals
			PushBool(value) => stack.push(parser::Constant::Bool(value)),
			PushInt(value) => stack.push(parser::Constant::Int(value)),
			PushStr(value) => stack.push(parser::Constant::Str(value)),

			// Keywords
			End => todo!(),
			If => todo!(),
			Elif => todo!(),
			Else => todo!(),
			While => todo!(),
			Let => todo!(),
			Then => todo!(),
			Do => todo!(),
			In => todo!(),
			Def(name) => todo!(),
			FunctionArgument(ty) => todo!(),
			Var(name) => todo!(),
			Struct(name) => todo!(),
			StructMember(name, ty) => todo!(),
			Enum(name) => todo!(),
			EnumVariant(name, tag) => todo!(),
			Include(path) => todo!(),
			DashDash => todo!(),

			// Operators
			Print => todo!(),
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
			Call(id) => todo!(),
		}
	}

	if stack.len() > 1 {
		return Err("Unhandled data in constant evaluation!".to_string());
	}

	stack
		.pop()
		.ok_or("Code does not evaluate to any value!".to_string())
}
