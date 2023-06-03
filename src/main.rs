mod codegen;
mod common;
mod ir;
mod irgen;
mod parse;
mod pass;
mod tokenize;
mod ty;
use common::Expected;
use inkwell::context::Context;
use std::fs::File;
use std::io::{self, Read, Write};

fn usage() {
  eprintln!("try-rust [-ll] [-o<path>] <file>")
}

fn read_file(path: &str) -> Expected<String> {
  if path == "-" {
    let input = io::stdin()
      .lines()
      .into_iter()
      .collect::<Result<Vec<_>, _>>();
    match input {
      Ok(v) => Ok(v.join("\n")),
      Err(_) => err!("failed to read from stdin"),
    }
  } else {
    let mut f = File::open(path).or(err!("file not found"))?;
    let mut input = String::new();
    match f.read_to_string(&mut input) {
      Ok(_) => Ok(input),
      Err(_) => err!("something went wrong reading the file"),
    }
  }
}

fn write_to_file(path: &str, body: &str) -> Expected<()> {
  if path == "-" {
    println!("{}", body);
  } else {
    let mut file = File::create(path).or(err!("failed to open file"))?;
    writeln!(file, "{}", body).or(err!("failed to write to file"))?;
  }
  Ok(())
}

fn main() -> Expected<()> {
  let mut target_ll = false;
  let mut output_path = String::from("-");
  let mut input_path = String::new();
  for arg in std::env::args().skip(1) {
    if arg == "--help" {
      usage();
      return Ok(());
    } else if arg == "-ll" {
      target_ll = true;
    } else if arg.starts_with("-o") {
      output_path = arg[2..].to_string();
    } else if arg.starts_with('-') && arg.len() > 1 {
      usage();
      return err!("unknown argument");
    } else {
      input_path = arg;
    }
  }

  let input = read_file(&input_path)?;
  let it = tokenize::Tokenizer::new(&input);
  let toplevels = parse::parse(it)?;

  let body = if target_ll {
    let context = Context::create();
    let module = codegen::ll::CodeGen::new(&context).codegen(toplevels)?;
    module.to_string()
  } else {
    let module = irgen::IRGen::new("mod".to_string()).irgen(toplevels)?;
    let module = pass::DeadCodeElimination::new(module).run();
    format!("{}", codegen::Target::C(module))
  };

  write_to_file(&output_path, &body)?;
  Ok(())
}
