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

	if let Err(err) = interpret(path) {
		eprintln!("Error: {}", err);
	}
}

fn interpret(path: String) -> Result<(), String> {
	let source = std::fs::read_to_string(path).or_else(|err| Err(format!("{}", err)))?;
	let code = parser::parse(source.chars().peekable())?;
	let typechecked = typer::typecheck(code)?;
	let program = compiler::compile(typechecked)?;
	evaluator::evaluate(program)?;
	Ok(())
}
