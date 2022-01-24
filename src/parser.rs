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
