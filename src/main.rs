mod evaluator;
mod parser;

fn main() {
	let ir = parser::parse(
		r#"
include "std.reko"

const X 5;
const Y X 2 *;
const MY-CONST Y 5 +;
const MESSAGE 
	"Evaluating `MESSAGE` constant!" print
	"Hello Mx. Ramble!"
end
#var x MY-CONST 2 *;

struct Foo
	activated bool
	skeggles  int
	name      str
	pointer   * int
end

def Foo.new
	bool    # activated
	int     # skeggles
	str     # name
	* int   # pointer
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

def Direction.str 
	int # direction
	--
	str
do
	if dup Direction.Up = then
		drop
		"Up"
	elif dup Direction.Down = then
		drop
		"Down"
	elif dup Direction.Left = then
		drop
		"Left"
	elif dup Direction.Right = then
		drop
		"Right"
	else
		drop
		""
	end
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
		MESSAGE print
	end
end"#
			.chars()
			.peekable(),
	);

	if let Err(err) = ir {
		eprintln!("Error: {}", err);
	}
}
