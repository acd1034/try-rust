use std::fs;
use std::io;
use std::num;

// ----- Combine error types -----

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

// ----- err macro -----

macro_rules! err {
  ($x:expr) => {
    Err(concat!($x, " [", file!(), ":", line!(), "]"))
  };
}

fn try_err() -> Result<i64, &'static str> {
  err!("file not exists")
}

// ----- print hyperlink -----

fn cat_literal() {
  println!(
    "[\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\]",
    "http://example.com", "Hyperlink"
  );
}

// ----- Add method to build-in types -----

pub trait Scream {
  fn scream(&self);
}

impl Scream for i32 {
  fn scream(&self) {
    println!("I am {}!", self);
  }
}

fn try_scream_i32() {
  42.scream();
}
