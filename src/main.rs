mod evaluator;
mod parser;

fn main() {
	let ir = parser::parse(
		r#"
include "std.reko"

const X 5;
const Y X 2 *;
const MY-CONST Y 5 +;
#var x MY-CONST 2 *;

struct Foo
	activated bool
	skeggles  int
	name      str
	pointer   * int
end

def Foo.new
	bool
	int
	str
	* int
	-- 
	Foo
do
	# values already pushed onto the stack
end

enum Direction
	Up
	Down
	Left
	Right
end

def main1 do
	true 5 "Hello" 0 Foo.new
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
