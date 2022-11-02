type Expected<T> = Result<T, &'static str>;

fn read_line() -> Expected<String> {
  let mut input = String::new();
  std::io::stdin()
    .read_line(&mut input)
    .map_err(|_| "failed to read line")?;
  Ok(input.trim_end().to_string())
}

fn include3(x: i64) -> bool {
  if x == 0 {
    false
  } else if x % 10 == 3 {
    true
  } else {
    include3(x / 10)
  }
}

fn solve() -> Expected<()> {
  let input = read_line()?;
  let n = input.parse::<i64>().map_err(|_| "failed to read n")?;
  for x in 1..=n {
    if x % 3 == 0 || include3(x) {
      print!(" {}", x)
    }
  }
  Ok(())
}

fn main() {
  match solve() {
    Ok(()) => {}
    Err(msg) => println!("{}", msg),
  }
}
