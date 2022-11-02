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

struct Nabeatsu {
  a: i64,
  b: i64,
}

impl Nabeatsu {
  // 初期化を行います。
  fn new(b: i64) -> Nabeatsu {
    Nabeatsu { a: 1, b: b }
  }
}

impl Iterator for Nabeatsu {
  type Item = i64;
  fn next(&mut self) -> Option<i64> {
    loop {
      if self.a > self.b {
        return None;
      } else if self.a % 3 == 0 || include3(self.a) {
        let x = self.a;
        self.a += 1;
        return Some(x);
      } else {
        self.a += 1;
      }
    }
  }
}

fn solve() -> Expected<()> {
  let input = read_line()?;
  let n = input.parse::<i64>().map_err(|_| "failed to read n")?;
  for x in Nabeatsu::new(n) {
    print!(" {}", x)
  }
  Ok(())
}

fn main() {
  match solve() {
    Ok(()) => {}
    Err(msg) => println!("{}", msg),
  }
}
