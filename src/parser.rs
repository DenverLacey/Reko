use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct Tokenizer<'a> {
	source: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
	pub fn new(source: Peekable<Chars<'a>>) -> Self {
		Self { source }
	}

	pub fn next(&mut self) -> Option<Token> {
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
			_ => Token {
				kind: TokenKind::Ident(string),
			},
		})
	}
}

#[derive(Debug)]
pub struct Token {
	kind: TokenKind,
}

#[derive(Debug)]
pub enum TokenKind {
	// Literals
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

	// Operators
	Print,
	Plus,
	Dash,
	Star,
	Slash,
	Eq,
}

type Chunk = Vec<Token>;
type Chunks = Vec<Chunk>;

pub fn chunkify<'a>(t: &mut Tokenizer<'a>) -> Result<Chunks, String> {
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
pub struct Parser {
	scopes: Vec<Scope>,
	global: Scope,
	next_var_id: usize,
}

impl Parser {
	pub fn new() -> Self {
		Self {
			scopes: Default::default(),
			global: Scope::new(ScopeKind::Global),
			next_var_id: 0,
		}
	}

	fn add_constant(&mut self, name: &String, constant: Constant) -> Result<(), String> {
		println!("ADD CONSTANT!");

		let scope = if self.scopes.is_empty() {
			&mut self.global
		} else {
			self.scopes.last_mut().unwrap()
		};

		let previous = scope.values.insert(name.clone(), Value::Constant(constant));
		if previous.is_some() {
			return Err(format!("Redeclaration of constant `{}`", name));
		}

		Ok(())
	}

	fn add_variable(&mut self, name: &String) -> Result<(), String> {
		println!("ADD VARIABLE#{}", self.next_var_id);

		let scope = if self.scopes.is_empty() {
			&mut self.global
		} else {
			self.scopes.last_mut().unwrap()
		};

		let previous = scope.values.insert(
			name.clone(),
			Value::Variable(Variable {
				index: self.next_var_id,
			}),
		);
		if previous.is_some() {
			return Err(format!("Redeclaration of constant `{}`", name));
		}

		self.next_var_id += 1;

		Ok(())
	}

	fn get_value(&self, name: &String) -> Option<&Value> {
		let mut iter = self.scopes.iter();
		while let Some(scope) = iter.next_back() {
			if let Some(c) = scope.values.get(name) {
				return Some(c);
			}
		}

		self.global.values.get(name)
	}
}

#[derive(Debug)]
struct Scope {
	kind: ScopeKind,
	values: HashMap<String, Value>,
}

impl Scope {
	fn new(kind: ScopeKind) -> Self {
		Self {
			kind,
			values: Default::default(),
		}
	}
}

#[derive(Debug, PartialEq)]
enum ScopeKind {
	Global,
	If,
	Elif,
	Else,
	While,
	Let,
	Def,
	Var,
	Const,
	Struct,
	Enum,
}

#[derive(Debug)]
enum Value {
	Constant(Constant),
	Variable(Variable),
}

#[derive(Debug)]
enum Constant {
	Bool(bool),
	Int(i64),
	Str(String),
}

#[derive(Debug)]
struct Variable {
	index: usize,
}

#[derive(Debug)]
pub struct FIR {
	kind: FIRKind,
}

#[derive(Debug)]
pub enum FIRKind {
	// Literals
	Ident(String),
	Int(i64),
	Str(String),

	// Keywords
	End,
	If,
	Elif,
	Else,
	While,
	Let(Vec<(bool, String)>),
	Then,
	Do,
	In,
	Def,
	Var,
	Struct,
	StructMember(String, TypeSignature),
	Enum,
	Include(String),

	// Operators
	Print,
	Plus,
	Dash,
	Star,
	Slash,
	Eq,
	LoadVar(usize),
}

#[derive(Debug)]
pub enum TypeSignature {
	Bool,
	Int,
	Str,
	Ptr(Box<TypeSignature>),
}

struct Queued {
	chunk: Chunk,
	fir: Vec<FIR>,
	cursor: usize,
}

impl Queued {
	fn new(chunk: Chunk) -> Self {
		Queued {
			chunk,
			fir: Default::default(),
			cursor: 0,
		}
	}

	fn finished(&self) -> bool {
		self.cursor == self.chunk.len()
	}

	fn next(&mut self) -> Option<&Token> {
		if self.cursor >= self.chunk.len() {
			return None;
		}
		let n = &self.chunk[self.cursor];
		self.cursor += 1;
		Some(n)
	}
}

impl Parser {
	pub fn parse(&mut self, chunks: Chunks) -> Result<Vec<Vec<FIR>>, String> {
		// @NOTE:
		// Maybe we should do a small prepass to sort out the chunks so that we
		// don't have to suspend parsing
		//
		let mut queued: Vec<_> = chunks.into_iter().map(|chunk| Queued::new(chunk)).collect();

		while queued.iter().any(|q| !q.finished()) {
			let mut i = 0;
			let mut made_progess = false;

			while i < queued.len() {
				{
					let q = &mut queued[i];
					if q.finished() {
						break;
					}

					self.try_parse(q)?;
				}

				if queued[i].finished() {
					made_progess = true;
					let q = queued.remove(i);
					queued.push(q);
				} else {
					i += 1;
				}
			}

			if !made_progess {
				return Err("No progess made while parsing!".to_string()); // Make better error message
			}
		}

		Ok(
			queued
				.into_iter()
				.filter_map(|p| if p.fir.is_empty() { None } else { Some(p.fir) })
				.collect(),
		)
	}

	fn try_parse(&mut self, queued: &mut Queued) -> Result<(), String> {
		while queued.cursor < queued.chunk.len() {
			let kind = &queued.chunk[queued.cursor].kind;
			queued.cursor += 1;

			match kind {
				// Literals
				TokenKind::Ident(ident) => {
					if let Some(value) = self.get_value(ident) {
						match value {
							Value::Constant(constant) => match constant {
								Constant::Bool(value) => todo!("implement booleans"),
								// queued.fir.push(FIR {
								// 	kind: FIRKind::Bool(value),
								// }),
								Constant::Int(value) => queued.fir.push(FIR {
									kind: FIRKind::Int(*value),
								}),
								Constant::Str(value) => queued.fir.push(FIR {
									kind: FIRKind::Str(value.clone()),
								}),
							},
							Value::Variable(variable) => queued.fir.push(FIR {
								kind: FIRKind::LoadVar(variable.index),
							}),
						}
					} else {
						return Ok(());
					}
				}
				TokenKind::Int(int) => queued.fir.push(FIR {
					kind: FIRKind::Int(*int),
				}),
				TokenKind::Str(string) => queued.fir.push(FIR {
					kind: FIRKind::Str(string.clone()),
				}),

				// Keywords
				TokenKind::End => {
					let popped = self
						.scopes
						.pop()
						.ok_or("Encountered `end` when there is no scope to end.".to_string())?;
					println!("Popped {:?}", popped);
					queued.fir.push(FIR { kind: FIRKind::End });
				}
				TokenKind::If => {
					self.scopes.push(Scope::new(ScopeKind::If));
					queued.fir.push(FIR { kind: FIRKind::If });
				}
				TokenKind::Elif => {
					let previous_scope = self
						.scopes
						.pop()
						.ok_or("Encountered `elif` without a parent `if`.".to_string())?;

					if previous_scope.kind != ScopeKind::If && previous_scope.kind != ScopeKind::Elif {
						return Err("Encountered `elif` without a parent `if`.".to_string());
					}

					self.scopes.push(Scope::new(ScopeKind::Elif));
					queued.fir.push(FIR {
						kind: FIRKind::Elif,
					});
				}
				TokenKind::Else => {
					let previous_scope = self
						.scopes
						.pop()
						.ok_or("Encountered `else` without a parent `if`.".to_string())?;

					if previous_scope.kind != ScopeKind::If && previous_scope.kind != ScopeKind::Elif {
						return Err("Encountered `else` without a parent `if`.".to_string());
					}

					self.scopes.push(Scope::new(ScopeKind::Else));
					queued.fir.push(FIR {
						kind: FIRKind::Else,
					});
				}
				TokenKind::While => {
					self.scopes.push(Scope::new(ScopeKind::While));
					queued.fir.push(FIR {
						kind: FIRKind::While,
					});
				}
				TokenKind::Let => todo!("implement parsing let expressions"),
				TokenKind::Then => queued.fir.push(FIR {
					kind: FIRKind::Then,
				}),
				TokenKind::Do => queued.fir.push(FIR { kind: FIRKind::Do }),
				TokenKind::In => queued.fir.push(FIR { kind: FIRKind::In }),
				TokenKind::Def => {
					self.scopes.push(Scope::new(ScopeKind::Def));
					queued.fir.push(FIR { kind: FIRKind::Def });

					if queued.cursor < queued.chunk.len() {
						let kind = &queued.chunk[queued.cursor].kind;
						queued.cursor += 1;
						if let TokenKind::Ident(ident) = kind {
							queued.fir.push(FIR {
								kind: FIRKind::Ident(ident.clone()),
							});
						} else {
							return Err("Expected an identifier after `def` keyword.".to_string());
						}
					} else {
						return Err("Expected an identifier after `def` keyword.".to_string());
					}
				}
				TokenKind::Var => {
					queued.fir.push(FIR { kind: FIRKind::Var });

					if queued.cursor < queued.chunk.len() {
						let kind = &queued.chunk[queued.cursor].kind;
						queued.cursor += 1;
						if let TokenKind::Ident(ident) = kind {
							self.add_variable(ident)?;
							queued.fir.push(FIR {
								kind: FIRKind::Ident(ident.clone()),
							});
						} else {
							return Err("Expected an identifier after `var` keyword.".to_string());
						}
					} else {
						return Err("Expected an identifier after `var` keyword.".to_string());
					}

					self.scopes.push(Scope::new(ScopeKind::Var));
				}
				TokenKind::Const => {
					// @TODO
					// Actually evaluate the value of the constant
					//
					let c = Constant::Int(0);

					match queued.next() {
						Some(Token {
							kind: TokenKind::Ident(name),
						}) => self.add_constant(&name, c)?,
						_ => return Err("Expected name of constant after `const` keyword.".to_string()),
					}

					// @HACK
					// Skipping everything up until the end of the constant declaration
					//
					while let Some(t) = queued.next() {
						println!("Skipping {:?}", t);
						if matches!(t.kind, TokenKind::End) {
							break;
						}
					}
				}
				TokenKind::Struct => todo!("implement parsing struct declarations"),
				TokenKind::Enum => {
					self.scopes.push(Scope::new(ScopeKind::Enum));
					queued.fir.push(FIR {
						kind: FIRKind::Enum,
					});
				}
				TokenKind::Include => {
					let path_kind = &queued.chunk[queued.cursor].kind;
					queued.cursor += 1;
					if let TokenKind::Str(path) = path_kind {
						queued.fir.push(FIR {
							kind: FIRKind::Include(path.clone()),
						});
					}
				}

				// Operators
				TokenKind::Print => queued.fir.push(FIR {
					kind: FIRKind::Print,
				}),
				TokenKind::Plus => queued.fir.push(FIR {
					kind: FIRKind::Plus,
				}),
				TokenKind::Dash => queued.fir.push(FIR {
					kind: FIRKind::Dash,
				}),
				TokenKind::Star => queued.fir.push(FIR {
					kind: FIRKind::Star,
				}),
				TokenKind::Slash => queued.fir.push(FIR {
					kind: FIRKind::Slash,
				}),
				TokenKind::Eq => queued.fir.push(FIR { kind: FIRKind::Eq }),
			}
		}

		Ok(())
	}
}
