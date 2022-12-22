mod codegen;
mod parse;
mod tokenize;
use inkwell::context::Context;
use tokenize::Expected;

fn compile(s: &str) -> Expected<String> {
  let it = tokenize::Tokenizer::new(s);
  let ast = parse::parse(it)?;

  let context = Context::create();
  let codegen = codegen::CodeGen {
    context: &context,
    module: context.create_module("main"),
    builder: context.create_builder(),
  };
  codegen.codegen(ast)
}

fn main() {
  match std::env::args().nth(1) {
    Some(arg) => match compile(arg.as_str()) {
      Ok(ir) => println!("{}", ir),
      Err(msg) => {
        eprintln!("error: {}", msg);
      }
    },
    None => eprintln!("error: preprocess: invalid number of arguments"),
  }
}
