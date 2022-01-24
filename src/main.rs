mod parser;

fn main() {
	let mut t = parser::Tokenizer::new(
		r#"
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

	while let Some(token) = t.next() {
		println!("{:?}", token);
	}
}
