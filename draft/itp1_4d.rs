type Expected<T> = Result<T, &'static str>;

fn read_line() -> Expected<String> {
  let mut input = String::new();
  std::io::stdin()
    .read_line(&mut input)
    .map_err(|_| "failed to read line")?;
  Ok(input.trim_end().to_string())
}

fn read_expr() -> Expected<(i64, i64, i64)> {
  read_line()?;
  let input = read_line()?;
  let nums: Vec<_> = input
    .split_whitespace()
    .map(|s| s.parse::<i64>().unwrap())
    .collect();
  // println!("{:?}", nums);
  let min = nums.iter().min().ok_or("faild to calculate min")?;
  let max = nums.iter().max().ok_or("faild to calculate max")?;
  let sum = nums.iter().fold(0, |x, y| x + y);
  Ok((min.clone(), max.clone(), sum))
}

fn main() {
  match read_expr() {
    Ok((min, max, sum)) => println!("{} {} {}", min, max, sum),
    Err(msg) => println!("{}", msg),
  }
}
