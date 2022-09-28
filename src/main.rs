fn parse(s: &str) -> Result<i32, &str> {
  s.parse::<i32>().map_err(|_| "Failed to convert")
}

fn main() {
  assert!(parse("42") == Ok(42));
}
