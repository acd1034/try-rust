use std::io::stdin;

fn read_expr() -> Option<(i64, i64, i64)> {
  let mut input = String::new();
  stdin().read_line(&mut input).ok()?;
  input.trim().parse::<i64>().ok()?;
  let mut input = String::new();
  stdin().read_line(&mut input).ok()?;
  let nums: Vec<_> = input
    .split_whitespace()
    .map(|s| s.trim().parse::<i64>().unwrap())
    .collect();
  // println!("{:?}", nums);
  let min = nums.iter().min()?;
  let max = nums.iter().max()?;
  let sum = nums.iter().fold(0, |x, y| x + y);
  Some((min.clone(), max.clone(), sum))
}

fn main() {
  if let Some((min, max, sum)) = read_expr() {
    println!("{} {} {}", min, max, sum)
  }
}
