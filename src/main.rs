mod codegen;
mod parse;
mod tokenize;
use inkwell::context::Context;
use tokenize::Expected;

fn compile(s: &str) -> Expected<String> {
  let it = tokenize::Tokenizer::new(s);
  let functions = parse::parse(it)?;

  let context = Context::create();
  let codegen = codegen::CodeGen {
    context: &context,
    module: context.create_module("mod"),
  };
  codegen.codegen(functions)
}

fn main() -> Expected<()> {
  let arg = std::env::args()
    .nth(1)
    .ok_or("main: invalid number of arguments")?;
  let ir = compile(&arg)?;
  println!("{}", ir);
  Ok(())
}
