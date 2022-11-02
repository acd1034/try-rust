type Expected<T> = Result<T, &'static str>;

fn read_line() -> Expected<String> {
  let mut input = String::new();
  std::io::stdin()
    .read_line(&mut input)
    .map_err(|_| "failed to read line")?;
  Ok(input.trim_end().to_string())
}

fn checker_line(s: &str, size: usize) -> String {
  let tail = if size % 2 == 1 {
    format!("{}\n", &s[..1])
  } else {
    format!("\n")
  };
  std::iter::repeat(s)
    .take(size / 2)
    .fold(String::new(), |x, y| x + y)
    + &tail
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
    let l1 = checker_line("#.", vec[1]);
    let l2 = checker_line(".#", vec[1]);
    let mut sum = String::new();
    for _ in 0..vec[0] / 2 {
      sum += &l1;
      sum += &l2;
    }
    if vec[0] % 2 == 1 {
      sum += &l1;
    }
    Ok(sum)
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
