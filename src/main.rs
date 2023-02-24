mod codegen;
mod common;
mod irgen;
mod parse;
mod tokenize;
use common::Expected;
use inkwell::context::Context;
use std::fs::File;
use std::io::{self, Read};

fn read_file(path: String) -> Expected<String> {
  if path == "-" {
    let input = io::stdin()
      .lines()
      .into_iter()
      .collect::<Result<Vec<_>, _>>();
    match input {
      Ok(v) => Ok(v.join("\n")),
      Err(_) => Err("failed to read from stdin"),
    }
  } else {
    let mut f = File::open(path).map_err(|_| "file not found")?;
    let mut input = String::new();
    match f.read_to_string(&mut input) {
      Ok(_) => Ok(input),
      Err(_) => Err("something went wrong reading the file"),
    }
  }
}

fn main() -> Expected<()> {
  let mut use_inkwell = false;
  let mut path = String::new();
  for arg in std::env::args().skip(1) {
    match arg.as_str() {
      "-inkwell" => {
        use_inkwell = true;
      }
      _ => {
        path = arg;
      }
    }
  }

  let input = read_file(path)?;
  let it = tokenize::Tokenizer::new(&input);
  let funs = parse::parse(it)?;

  let code = if use_inkwell {
    let context = Context::create();
    codegen::CodeGen::new(&context).codegen(funs)?
  } else {
    format!("{}", irgen::irgen(funs)?)
  };

  println!("{}", code);
  Ok(())
}
