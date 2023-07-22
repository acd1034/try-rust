use inkwell;
use ir1;
use ll;
use parser::*;
use std::fs::File;
use std::io::{self, Read, Write};

// ----- read_command_line_option -----

enum Target {
  LL,
  IR1,
}

struct CommandLineOption {
  target: Target,
  input_path: String,
  output_path: String,
}

fn show_usage() {
  eprintln!("try-rust [-ll|-ir1] [-o<path>] <file>")
}

fn read_command_line_option() -> common::Expected<CommandLineOption> {
  let mut target = Target::LL;
  let mut input_path = String::new();
  let mut output_path = String::from("-");

  for arg in std::env::args().skip(1) {
    if arg == "--help" {
      show_usage();
      break;
    } else if arg == "-ll" {
      target = Target::LL;
    } else if arg == "-ir1" {
      target = Target::IR1;
    } else if arg.starts_with("-o") {
      output_path = arg[2..].to_string();
    } else if arg.starts_with('-') && arg.len() > 1 {
      show_usage();
      return err!("unknown argument");
    } else {
      input_path = arg;
    }
  }

  Ok(CommandLineOption {
    target,
    input_path,
    output_path,
  })
}

// ----- read_file, write_to_file -----

fn read_file(path: &str) -> common::Expected<String> {
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

fn write_to_file(path: &str, body: &str) -> common::Expected<()> {
  if path == "-" {
    println!("{}", body);
  } else {
    let mut file = File::create(path).or(err!("failed to open file"))?;
    writeln!(file, "{}", body).or(err!("failed to write to file"))?;
  }
  Ok(())
}

// ----- main -----

fn main() -> common::Expected<()> {
  let opt = read_command_line_option()?;

  let toplevels = {
    let input = read_file(&opt.input_path)?;
    let it = tokenize::Tokenizer::new(&input);
    parse::parse(it)?
  };

  let body = match opt.target {
    Target::LL => {
      let context = inkwell::context::Context::create();
      let module = ll::CodeGen::new(&context).codegen(toplevels)?;
      module.to_string()
    }
    Target::IR1 => {
      let module = ir1::irgen::IRGen::new("mod".to_string()).irgen(toplevels)?;
      let module = ir1::pass::DeadCodeElimination::new(module).run();
      format!("{}", ir1::codegen::Target::C(module))
    }
  };

  write_to_file(&opt.output_path, &body)?;
  Ok(())
}
