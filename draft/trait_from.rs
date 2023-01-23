use std::fs;
use std::io;
use std::num;

enum CliError {
  IoError(io::Error),
  ParseError(num::ParseIntError),
}

impl From<io::Error> for CliError {
  fn from(error: io::Error) -> Self {
    CliError::IoError(error)
  }
}

impl From<num::ParseIntError> for CliError {
  fn from(error: num::ParseIntError) -> Self {
    CliError::ParseError(error)
  }
}

fn open_and_parse_file(file_name: &str) -> Result<i32, CliError> {
  let mut contents = fs::read_to_string(&file_name)?;
  let num: i32 = contents.trim().parse()?;
  Ok(num)
}

enum Message {
  Quit,
  Write(String),
  Move { x: i32, y: i32 },
}

fn print_msg() {
  let m = Message::Move { x: 0, y: 1 };
  match m {
    Message::Quit => println!("Quit!"),
    Message::Write(s) => println!("Write: {}", s),
    Message::Move { x, y } => println!("Move: ({}, {})", x, y),
  }
}

macro_rules! err {
  ($x:expr) => {
    Err(concat!(file!(), ":", line!(), "\n", $x))
  };
}

fn cat_literal() -> Result<i64, &'static str> {
  err!("file not exists")
}
