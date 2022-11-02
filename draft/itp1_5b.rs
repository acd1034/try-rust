type Expected<T> = Result<T, &'static str>;

fn read_line() -> Expected<String> {
  let mut input = String::new();
  std::io::stdin()
    .read_line(&mut input)
    .map_err(|_| "failed to read line")?;
  Ok(input.trim_end().to_string())
}

fn read_expr() -> Expected<String> {
  let input = read_line()?;
  let vec = input
    .split_whitespace()
    .map(|s| s.parse::<usize>().unwrap())
    .collect::<Vec<_>>();
  if vec == vec![0, 0] {
    Err("EOF")
  } else {
    let front = format!("{:#<width$}\n", "", width = vec[1]);
    let back = front.clone();
    let s = format!("#{:.<width$}#\n", "", width = vec[1] - 2);
    let sum = std::iter::repeat(s)
      .take(vec[0] - 2)
      .fold(front, |x, y| x + &y);
    Ok(sum + &back)
  }
}

fn main() {
  loop {
    match read_expr() {
      Ok(sum) => println!("{}", sum),
      Err("EOF") => break,
      Err(msg) => println!("{}", msg),
    }
  }
}
