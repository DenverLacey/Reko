mod parser;

fn main() {
	let ir = parser::parse(
		r#"
include "std.reko"

const MY-CONST 23;
#var x MY-CONST 2 *;

struct Foo
	activated bool
	skeggles  int
	name      str
	pointer   * int
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
			MY-CONST 10 + # This is a comment
		else
			7 2 -
		end 1 =
	do
		"HELLO!" print
	end
end"#
			.chars()
			.peekable(),
	);

	if let Err(err) = ir {
		eprintln!("Error: {}", err);
	}
}
