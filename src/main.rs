mod parser;

fn main() {
	let mut t = parser::Tokenizer::new(
		r#"
if 1 1 = then 
	😎 🤩 + # This is a comment
else
	1 2 -
end"#
			.chars()
			.peekable(),
	);
	println!("{:?}", t);

	while let Some(token) = t.next() {
		println!("{:?}", token);
	}
}
