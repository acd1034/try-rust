type Expected<T> = Result<T, &'static str>;

fn read_line() -> Expected<String> {
  let mut input = String::new();
  std::io::stdin()
    .read_line(&mut input)
    .map_err(|_| "failed to read line")?;
  Ok(input.trim_end().to_string())
}

fn solve() -> Expected<()> {
  read_line()?;
  let input = read_line()?;
  // let nums: Vec<_> = input.split_whitespace().collect();
  // let it = nums.iter().rev();
  let mut it = input.split_whitespace().rev();
  print!("{}", it.next().unwrap());
  for x in it {
    print!(" {}", x)
  }
  Ok(())
}

fn main() {
  match solve() {
    Ok(()) => println!(""),
    Err(msg) => println!("{}", msg),
  }
}
