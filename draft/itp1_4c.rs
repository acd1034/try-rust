use std::io::stdin;

fn read_expr() -> Option<i32> {
  let mut input = String::new();
  stdin().read_line(&mut input).ok()?;
  let toks: Vec<_> = input.split_whitespace().collect();
  let x = toks[0].parse::<i32>().ok()?;
  let y = toks[2].trim().parse::<i32>().ok()?;
  match toks[1] {
    "+" => Some(x + y),
    "-" => Some(x - y),
    "*" => Some(x * y),
    "/" => Some(x / y),
    _ => None,
  }
}

fn main() {
  loop {
    match read_expr() {
      Some(x) => println!("{}", x),
      None => break,
    }
  }
}
