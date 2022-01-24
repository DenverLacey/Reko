mod parser;

fn main() {
	let mut t = parser::Tokenizer::new("😎 🤩 +".chars().peekable());
	println!("{:?}", t);

	while let Some(token) = t.next() {
		println!("{:?}", token);
	}
}
