mod parser;

fn main() {
	let mut t = parser::Tokenizer::new(
		r#"
include "std.reko"

const MY_CONST 23;
var x int MY_CONST 2 *;

struct Point
	x int
	y int
end

enum Direction
	Up
	Down
	Left
	Right
end

def main do
	while
		if 1 1 = then
			😎 🤩 + # This is a comment
		else
			1 2 -
		end 1 =
	do
		"HELLO!" print
	end
end"#
			.chars()
			.peekable(),
	);

	let chunks = parser::chunkify(&mut t);
	match chunks {
		Ok(chunks) => println!("{:#?}", chunks),
		Err(err) => eprintln!("Error: {}", err),
	}
}
