mod parser;

fn main() {
	let mut t = parser::Tokenizer::new(
		r#"
include "std.reko"

const MY_CONST 23;
var x int MY_CONST 2 *;

#struct Point
#	x int
#	y int
#end

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
	if let Err(err) = chunks {
		eprintln!("Error: {}", err);
		return;
	}

	let chunks = chunks.unwrap();
	println!("{:#?}", chunks);

	let mut p = parser::Parser {};
	let fir = p.parse(chunks);
	if let Err(err) = fir {
		eprintln!("Error: {}", err);
		return;
	}

	let fir = fir.unwrap();
	println!("{:#?}", fir);
}
