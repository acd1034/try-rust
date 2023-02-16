mod codegen;
mod parse;
mod tokenize;
use inkwell::context::Context;
use tokenize::Expected;

fn main() -> Expected<()> {
  let mut use_inkwell = false;
  let mut input = String::new();
  for arg in std::env::args().skip(1) {
    match arg.as_str() {
      "-inkwell" => {
        use_inkwell = true;
      }
      _ => {
        input = arg;
      }
    }
  }

  let it = tokenize::Tokenizer::new(&input);
  let funs = parse::parse(it)?;

  let code = if use_inkwell {
    let context = Context::create();
    codegen::CodeGen::new(&context).codegen(funs)?
  } else {
    todo!()
  };

  println!("{}", code);
  Ok(())
}
