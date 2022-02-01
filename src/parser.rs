use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

use crate::evaluator;

// @TODO:
// Implement out-of-order compilation.
// (For now we're simplifying the problem to make early progress and to give
// us context when we do)
//
pub fn parse<'a>(source: Peekable<Chars<'a>>) -> Result<IRChunks, String> {
	let mut tokenizer = Tokenizer::new(source);
	let chunks = chunkify(&mut tokenizer)?;
	println!("{:#?}", chunks);

	let mut parser = Parser::new();
	let mut ir = Vec::new();

	for chunk in chunks {
		let chunk_ir = parser.parse_chunk(chunk)?;
		if !chunk_ir.is_empty() {
			ir.push(chunk_ir);
		}
	}

	println!("{:#?}", parser);

	println!("{:#?}", ir);

	Ok(ir)
}

#[derive(Debug)]
struct Tokenizer<'a> {
	source: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
	fn new(source: Peekable<Chars<'a>>) -> Self {
		Self { source }
	}

	fn next(&mut self) -> Option<Token> {
		self.skip_whitespace();

		if let Some(&c) = self.source.peek() {
			if c == '"' {
				self.tokenize_string()
			} else if c == ';' {
				self.source.next();
				Some(Token {
					kind: TokenKind::End,
				})
			} else if c.is_ascii_digit() {
				self.tokenize_number()
			} else {
				self.tokenize_identifier_or_keyword()
			}
		} else {
			None
		}
	}

	fn skip_whitespace(&mut self) {
		while let Some(&c) = self.source.peek() {
			match c {
				'#' => self.skip_comment(),
				_ if !c.is_whitespace() => break,
				_ => {}
			}

			self.source.next();
		}
	}

	fn skip_comment(&mut self) {
		while let Some(_) = self.source.next_if(|&c| c != '\n') {}
	}

	fn tokenize_string(&mut self) -> Option<Token> {
		assert_eq!(
			'"',
			self
				.source
				.next()
				.expect("Tried to tokenize string but encountered EOF!")
		);

		// @TODO:
		// Handled escape sequences.
		//

		let mut string = String::new();
		while let Some(c) = self.source.next_if(|&c| c != '"') {
			string.push(c);
		}

		self.source.next(); // skip terminating `"`

		Some(Token {
			kind: TokenKind::Str(string),
		})
	}

	fn tokenize_number(&mut self) -> Option<Token> {
		let mut string = String::new();
		while let Some(c) = self.source.next_if(|&c| c.is_ascii_digit()) {
			string.push(c);
		}

		Some(Token {
			kind: TokenKind::Int(
				string
					.parse()
					.expect("This shouldn't fail because of the while loop checking `is_ascii_digit()`"),
			),
		})
	}

	fn tokenize_identifier_or_keyword(&mut self) -> Option<Token> {
		let mut string = String::new();
		while let Some(c) = self.source.next_if(|&c| !c.is_whitespace() && c != ';') {
			string.push(c);
		}

		Some(match string.as_str() {
			"true" => Token {
				kind: TokenKind::True,
			},
			"false" => Token {
				kind: TokenKind::False,
			},
			"end" => Token {
				kind: TokenKind::End,
			},
			"if" => Token {
				kind: TokenKind::If,
			},
			"elif" => Token {
				kind: TokenKind::Elif,
			},
			"else" => Token {
				kind: TokenKind::Else,
			},
			"while" => Token {
				kind: TokenKind::While,
			},
			"let" => Token {
				kind: TokenKind::Let,
			},
			"then" => Token {
				kind: TokenKind::Then,
			},
			"do" => Token {
				kind: TokenKind::Do,
			},
			"in" => Token {
				kind: TokenKind::In,
			},
			"def" => Token {
				kind: TokenKind::Def,
			},
			"var" => Token {
				kind: TokenKind::Var,
			},
			"const" => Token {
				kind: TokenKind::Const,
			},
			"struct" => Token {
				kind: TokenKind::Struct,
			},
			"enum" => Token {
				kind: TokenKind::Enum,
			},
			"include" => Token {
				kind: TokenKind::Include,
			},
			"--" => Token {
				kind: TokenKind::DashDash,
			},
			"dup" => Token {
				kind: TokenKind::Dup,
			},
			"over" => Token {
				kind: TokenKind::Over,
			},
			"drop" => Token {
				kind: TokenKind::Drop,
			},
			"print" => Token {
				kind: TokenKind::Print,
			},
			"+" => Token {
				kind: TokenKind::Plus,
			},
			"-" => Token {
				kind: TokenKind::Dash,
			},
			"*" => Token {
				kind: TokenKind::Star,
			},
			"/" => Token {
				kind: TokenKind::Slash,
			},
			"=" => Token {
				kind: TokenKind::Eq,
			},
			"!=" => Token {
				kind: TokenKind::Neq,
			},
			"<" => Token {
				kind: TokenKind::Lt,
			},
			">" => Token {
				kind: TokenKind::Gt,
			},
			_ => Token {
				kind: TokenKind::Ident(string),
			},
		})
	}
}

type Tokens = Peekable<std::vec::IntoIter<Token>>;

#[derive(Debug)]
struct Token {
	kind: TokenKind,
}

#[derive(Debug)]
enum TokenKind {
	// Literals
	True,
	False,
	Ident(String),
	Int(i64),
	Str(String),

	// Keywords
	End,
	If,
	Elif,
	Else,
	While,
	Let,
	Then,
	Do,
	In,
	Def,
	Var,
	Const,
	Struct,
	Enum,
	Include,
	DashDash,

	// Operators
	Dup,
	Over,
	Drop,
	Print,
	Plus,
	Dash,
	Star,
	Slash,
	Eq,
	Neq,
	Lt,
	Gt,
}

type Chunk = Vec<Token>;
type Chunks = Vec<Chunk>;

fn chunkify<'a>(t: &mut Tokenizer<'a>) -> Result<Chunks, String> {
	let mut chunks = Chunks::new();

	while let Some(token) = t.next() {
		use TokenKind::*;
		if !matches!(token.kind, Def | Var | Const | Struct | Enum | Include) {
			return Err(format!("{:?} cannot be at top level!", token));
		}

		let mut chunk = Chunk::new();

		if matches!(token.kind, Include) {
			let include_path = t
				.next()
				.ok_or("Expected a file path to include!".to_string())?;

			if !matches!(include_path.kind, Str(_)) {
				return Err("Expected a file path to include!".to_string());
			}

			chunk.push(token);
			chunk.push(include_path);
		} else {
			chunk.push(token);

			let mut num_expected_ends = 1;
			loop {
				if let Some(token) = t.next() {
					match token.kind {
						End => num_expected_ends -= 1,
						If | While | Def | Var | Const | Struct | Enum => num_expected_ends += 1,
						_ => {}
					}
					chunk.push(token);
				}

				if num_expected_ends == 0 {
					break;
				}
			}
		}

		chunks.push(chunk);
	}

	Ok(chunks)
}

#[derive(Debug)]
struct Parser {
	global: Scope,
	scopes: Vec<Scope>,
}

impl Parser {
	fn new() -> Self {
		Self {
			global: Scope::new(ScopeKind::Global),
			scopes: Default::default(),
		}
	}

	fn push_scope(&mut self, kind: ScopeKind) {
		self.scopes.push(Scope::new(kind));
	}

	fn pop_scope(&mut self) -> Option<Scope> {
		self.scopes.pop()
	}

	fn get_binding(&self, name: &String) -> Option<&Binding> {
		let mut iter = self.scopes.iter();
		while let Some(scope) = iter.next_back() {
			if let Some(value) = scope.bindings.get(name) {
				return Some(value);
			}
		}

		if let Some(value) = self.global.bindings.get(name) {
			Some(value)
		} else {
			None
		}
	}

	fn evaluate_constant(&mut self, tokens: &mut Tokens) -> Result<Constant, String> {
		// @TODO:
		// Actually evaluate the constant
		//
		let mut constant_chunk = Chunk::new();
		while let Some(t) = tokens.next() {
			if matches!(t.kind, TokenKind::End) {
				break;
			}
			constant_chunk.push(t);
		}

		let constant_ir = self.parse_chunk(constant_chunk)?;
		evaluator::constant_evaluate(constant_ir)
	}

	fn bind(&mut self, name: String, binding: Binding) -> Result<(), String> {
		let scope = if self.scopes.is_empty() {
			&mut self.global
		} else {
			self.scopes.last_mut().expect("We just checked is_empty")
		};

		if scope.bindings.contains_key(&name) {
			return Err(format!("Redeclared identifier `{}`", name));
		}

		scope.bindings.insert(name, binding);

		Ok(())
	}

	// fn bind_constant(&mut self, name: String, constant: Constant) -> Result<(), String> {
	// 	self.bind(name, Binding::Constant(constant))
	// }

	// fn bind_function(&mut self, name: String) -> Result<(), String> {
	// 	self.bind(name, Binding::Function)
	// }

	// fn bind_struct(&mut self, name: String) -> Result<(), String> {
	// 	self.bind(name, Binding::Struct)
	// }

	// fn bind_enum(&mut self, name: String) -> Result<(), String> {
	// 	self.bind(name, Binding::Enum)
	// }
}

impl Parser {
	fn parse_chunk(&mut self, chunk: Chunk) -> Result<IRChunk, String> {
		let mut generated = IRChunk::new();

		let mut iter = chunk.into_iter().peekable();
		while let Some(token) = iter.next() {
			use TokenKind::*;
			match token.kind {
				// Literals
				True => generated.push(IR {
					kind: IRKind::PushBool(true),
				}),
				False => generated.push(IR {
					kind: IRKind::PushBool(false),
				}),
				Ident(ident) => {
					match self
						.get_binding(&ident)
						.ok_or_else(|| format!("Unknown identifier `{}`", ident))?
					{
						Binding::Constant(constant) => match constant {
							Constant::Bool(value) => generated.push(IR {
								kind: IRKind::PushBool(*value),
							}),
							Constant::Int(value) => generated.push(IR {
								kind: IRKind::PushInt(*value),
							}),
							Constant::Str(value) => generated.push(IR {
								kind: IRKind::PushStr(value.clone()),
							}),
						},
						Binding::Variable(id) => todo!(),
						Binding::Function => generated.push(IR {
							kind: IRKind::Call(ident),
						}),
						Binding::Struct => todo!(),
						// Binding::Enum => todo!(),
					}
				}
				Int(value) => generated.push(IR {
					kind: IRKind::PushInt(value),
				}),
				Str(value) => generated.push(IR {
					kind: IRKind::PushStr(value),
				}),

				// Keywords
				End => {
					self
						.pop_scope()
						.ok_or("Unexpected `end` keyword. No blocks to end!")?;
					generated.push(IR { kind: IRKind::End });
				}
				If => generated.push(IR { kind: IRKind::If }),
				Elif => {
					let previous = self.pop_scope();
					if !matches!(
						previous,
						Some(Scope {
							kind: ScopeKind::If,
							bindings: _
						})
					) {
						return Err("`elif` block without a parent `if` block!".to_string());
					}
					generated.push(IR { kind: IRKind::Elif });
				}
				Else => {
					let previous = self.pop_scope();
					if !matches!(
						previous,
						Some(Scope {
							kind: ScopeKind::If,
							bindings: _
						})
					) {
						return Err("`else` block without a parent `if` block!".to_string());
					}
					self.push_scope(ScopeKind::Else);
					generated.push(IR { kind: IRKind::Else });
				}
				While => generated.push(IR {
					kind: IRKind::While,
				}),
				Let => todo!(),
				Then => {
					self.push_scope(ScopeKind::If);
					generated.push(IR { kind: IRKind::Then });
				}
				Do => {
					self.push_scope(ScopeKind::Def);
					generated.push(IR { kind: IRKind::Do });
				}
				In => todo!(),
				Def => {
					let ident = match iter.next() {
						Some(Token {
							kind: TokenKind::Ident(ident),
						}) => ident,
						_ => return Err("Expected an identifier after `def` keyword!".to_string()),
					};

					self.bind(ident.clone(), Binding::Function)?;

					generated.push(IR {
						kind: IRKind::Def(ident),
					});

					loop {
						match iter.peek() {
							Some(Token {
								kind: TokenKind::Do,
							}) => {
								generated.push(IR { kind: IRKind::Do });
								self.push_scope(ScopeKind::Def);
								iter.next(); // skip the do
								break;
							}
							Some(Token {
								kind: TokenKind::DashDash,
							}) => {
								generated.push(IR {
									kind: IRKind::DashDash,
								});
								iter.next(); // skip --
							}
							None => return Err("Unexpected EOF while parsing function!".to_string()),
							_ => {
								let arg_type_signature = self.parse_type_signature(&mut iter)?;
								generated.push(IR {
									kind: IRKind::FunctionArgument(arg_type_signature),
								});
							}
						}
					}
				}
				Var => todo!(),
				Const => {
					let ident = match iter.next() {
						Some(Token {
							kind: TokenKind::Ident(ident),
						}) => ident,
						_ => return Err("Expected an identifier after `const` keyword!".to_string()),
					};
					let value = self.evaluate_constant(&mut iter)?;
					self.bind(ident, Binding::Constant(value))?;
				}
				Struct => {
					let ident = match iter.next() {
						Some(Token {
							kind: TokenKind::Ident(ident),
						}) => ident,
						_ => return Err("Expected an identifier after `struct` keyword!".to_string()),
					};

					self.bind(ident.clone(), Binding::Struct)?;

					generated.push(IR {
						kind: IRKind::Struct(ident),
					});

					loop {
						match iter.next() {
							None => return Err("Unexpected EOF while parsing struct!".to_string()),
							Some(Token {
								kind: TokenKind::End,
							}) => break,
							Some(Token {
								kind: TokenKind::Ident(ident),
							}) => {
								let type_signature = self.parse_type_signature(&mut iter)?;
								generated.push(IR {
									kind: IRKind::StructMember(ident, type_signature),
								});
							}
							_ => return Err("Expected identifier of a struct field!".to_string()),
						}
					}
				}
				Enum => {
					let ident = match iter.next() {
						Some(Token {
							kind: TokenKind::Ident(ident),
						}) => ident,
						_ => return Err("Expected an identifier after `enum` keyword!".to_string()),
					};

					// self.bind(ident.clone(), Binding::Enum)?;

					// generated.push(IR {
					// 	kind: IRKind::Enum(ident),
					// });

					let mut variant_id = 0;
					loop {
						match iter.next() {
							None => return Err("Unexpected EOF while parsing enum!".to_string()),
							Some(Token {
								kind: TokenKind::End,
							}) => break,
							Some(Token {
								kind: TokenKind::Ident(variant),
							}) => {
								self.bind(
									format!("{}.{}", ident, variant),
									Binding::Constant(Constant::Int(variant_id)),
								)?;
							}
							// generated.push(IR {
							// 	kind: IRKind::EnumVariant(variant, variant_id),
							// }),
							_ => return Err("Expected identifier of an enum variant!".to_string()),
						}

						variant_id += 1;
					}
				}
				Include => match iter.next() {
					Some(Token {
						kind: TokenKind::Str(path),
					}) => generated.push(IR {
						kind: IRKind::Include(path),
					}),
					_ => return Err("Expected a path to include after `include` keyword!".to_string()),
				},
				DashDash => generated.push(IR {
					kind: IRKind::DashDash,
				}),

				// Operators
				Dup => generated.push(IR { kind: IRKind::Dup }),
				Over => generated.push(IR { kind: IRKind::Over }),
				Drop => generated.push(IR { kind: IRKind::Drop }),
				Print => generated.push(IR {
					kind: IRKind::Print,
				}),
				Plus => generated.push(IR { kind: IRKind::Add }),
				Dash => generated.push(IR {
					kind: IRKind::Subtract,
				}),
				Star => generated.push(IR {
					kind: IRKind::Multiply,
				}),
				Slash => generated.push(IR {
					kind: IRKind::Divide,
				}),
				Eq => generated.push(IR { kind: IRKind::Eq }),
				Neq => generated.push(IR { kind: IRKind::Neq }),
				Lt => generated.push(IR { kind: IRKind::Lt }),
				Gt => generated.push(IR { kind: IRKind::Gt }),
			}
		}

		Ok(generated)
	}

	fn parse_type_signature(&self, tokens: &mut Tokens) -> Result<TypeSignature, String> {
		match tokens.next() {
			Some(Token {
				kind: TokenKind::Ident(ident),
			}) => {
				if ident == "bool" {
					Ok(TypeSignature::Bool)
				} else if ident == "int" {
					Ok(TypeSignature::Int)
				} else if ident == "str" {
					Ok(TypeSignature::Str)
				} else {
					match self.get_binding(&ident) {
						Some(Binding::Struct) => Ok(TypeSignature::Struct(ident)),
						// Some(Binding::Enum) => Ok(TypeSignature::Enum(ident)),
						None => Err(format!("Undeclared identifier `{}`", ident)),
						_ => Err("Invalid type signature!".to_string()),
					}
				}
			}
			Some(Token {
				kind: TokenKind::Star,
			}) => Ok(TypeSignature::Ptr(Box::new(
				self.parse_type_signature(tokens)?,
			))),
			None => Err("Unexpected EOF while parsing type signature!".to_string()),
			_ => Err("Invalid type signature!".to_string()),
		}
	}
}

#[derive(Debug)]
struct Scope {
	kind: ScopeKind,
	bindings: HashMap<String, Binding>,
}

impl Scope {
	fn new(kind: ScopeKind) -> Self {
		Self {
			kind,
			bindings: Default::default(),
		}
	}
}

#[derive(Debug)]
enum ScopeKind {
	Global,
	Def,
	If,
	Elif,
	Else,
}

#[derive(Debug)]
enum Binding {
	Constant(Constant),
	Variable(usize),
	Function,
	Struct,
	// Enum,
}

#[derive(Debug)]
pub enum Constant {
	Bool(bool),
	Int(i64),
	Str(String),
}

pub type IRChunk = Vec<IR>;
pub type IRChunks = Vec<IRChunk>;

#[derive(Debug)]
pub struct IR {
	pub kind: IRKind,
}

#[derive(Debug)]
pub enum IRKind {
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
	Let,
	Then,
	Do,
	In,
	Def(String),
	FunctionArgument(TypeSignature),
	Var(String),
	Struct(String),
	StructMember(String, TypeSignature),
	Include(String),
	DashDash,

	// Operators
	Dup,
	Over,
	Drop,
	Print,
	Add,
	Subtract,
	Multiply,
	Divide,
	Eq,
	Neq,
	Lt,
	Gt,
	Call(String),
}

#[derive(Debug)]
pub enum TypeSignature {
	Bool,
	Int,
	Str,
	Ptr(Box<TypeSignature>),
	Struct(String),
	// Enum(String),
}
