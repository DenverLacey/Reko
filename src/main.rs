mod parser;

fn main() {
	let ir = parser::parse(
		r#"
include "std.reko"

const MY_CONST 23;
#var x MY_CONST 2 *;

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

#def main do
#	while
#		if 1 1 = then
#			MY_CONST 10 + # This is a comment
#		else
#			x 2 -
#		end 1 =
#	do
#		"HELLO!" print
#	end
#end"#
			.chars()
			.peekable(),
	);

	match ir {
		Ok(ir) => println!("{:#?}", ir),
		Err(err) => eprintln!("Error: {}", err),
	}
}
