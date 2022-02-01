mod compiler;
mod evaluator;
mod parser;
mod typer;

fn main() {
	let path = if let Some(path) = std::env::args().skip(1).next() {
		path
	} else {
		eprintln!("Error: Filepath to reko source file not provided!");
		return;
	};

	match std::fs::read_to_string(path) {
		std::io::Result::Ok(source) => match parser::parse(source.chars().peekable()) {
			Ok(code) => match typer::typecheck(code) {
				Ok(typechecked) => match compiler::compile(typechecked) {
					Ok(program) => {
						println!("{:#?}\n---------", program);
						if let Err(err) = evaluator::evaluate(program) {
							eprintln!("Error: {}", err);
						}
					}
					Err(err) => eprintln!("Error: {}", err),
				},
				Err(err) => println!("Error: {}", err),
			},
			Err(err) => eprintln!("Error: {}", err),
		},
		std::io::Result::Err(err) => {
			eprintln!("Error: {}", err);
		}
	}
}
