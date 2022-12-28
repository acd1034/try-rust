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
    module: context.create_module(""),
    builder: context.create_builder(),
  };
  codegen.codegen(ast)
}

fn main() -> Expected<()> {
  let arg = std::env::args()
    .nth(1)
    .ok_or("preprocess: invalid number of arguments")?;
  let ir = compile(&arg)?;
  println!("target triple = \"arm64-apple-macosx12.0.0\"\n{}", ir);
  Ok(())
}
