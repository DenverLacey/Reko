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
		while let Some(c) = self.source.next_if(|&c| !c.is_whitespace()) {
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
			"then" => Token {
				kind: TokenKind::Then,
			},
			"do" => Token {
				kind: TokenKind::Do,
			},
			"in" => Token {
				kind: TokenKind::In,
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
	Then,
	Do,
	In,
}
