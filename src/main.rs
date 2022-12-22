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

fn test(s: &str) {
  match compile(s) {
    Ok(n) => println!("{}", n),
    Err(msg) => println!("error: {}", msg),
  }
}

fn main() {
  // // primary
  // test("(1 + 2 + 3) * 4");
  // test("1 + 2 * (3 + 4 * 5 + 6) * 7 + 8");
  // test("1 * (2 + 3 * (4 + 5) * 6 + 7) * 8");
  // test("1 * _2 + 3)");
  // test("1 * (_ + 3)");
  // test("1 * (2 _ 3)");
  // test("1 * (2 + _)");
  // test("1 * (2 +  )");
  // test("1 * (2 + 3_");
  // test("1 * (2 + 3 ");

  // // expr
  // test("1 + 2 + 3 + 4");
  // test("1 + 2 - 3 + 4");
  // test("1 * 2 + 3 * 4 + 5 * 6");
  // test("1 * 2 - 6 / 3 + 4 * 5");
  // test("_ + 2");
  // test("1 _ 2");
  // test("1 + _");
  // test("1 +  ");

  // // term
  // test("1 * 2 * 3 * 4");
  // test("3 * 4 / 6 * 2");
  // test("_ * 2");
  // test("1 _ 2");
  // test("1 * _");
  // test("1 *  ");

  // num
  test("42");
  test("  123  ");
  test("  _  ");
  test("     ");

  // match compile("42") {
  //   Ok(n) => println!("{}", n),
  //   Err(msg) => println!("error: {}", msg),
  // }
}
