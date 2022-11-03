use std::cmp::Ordering;
type Expected<T> = Result<T, &'static str>;

fn read_line() -> Expected<String> {
  let mut input = String::new();
  std::io::stdin()
    .read_line(&mut input)
    .map_err(|_| "failed to read line")?;
  Ok(input.trim_end().to_string())
}

fn str2int(x: &str) -> i64 {
  match x {
    "S" => 0,
    "H" => 1,
    "C" => 2,
    _ => 3,
  }
}

fn cmp_tup(x: (i64, i64), y: (i64, i64)) -> Ordering {
  x.0.cmp(&y.0).then_with(|| x.1.cmp(&y.1))
}

fn solve() -> Expected<()> {
  read_line()?;
  let mut tups = std::io::stdin()
    .lines()
    .map(|s| {
      let st = s.unwrap();
      let mut it = st.split_ascii_whitespace();
      let c = str2int(it.next().unwrap());
      let n = it.next().unwrap().parse::<i64>().unwrap();
      (c, n)
    })
    .collect::<Vec<_>>();
  tups.sort_unstable_by(|&x, &y| cmp_tup(x, y));
  let mut it = tups.iter();
  let mut y = it.next();
  let int2str = ["S", "H", "C", "D"];
  for i in 0..4 {
    for j in 1..=13 {
      if &(i, j) == y.unwrap() {
        y = it.next();
      } else {
        println!("{} {}", int2str[i as usize], j);
      }
    }
  }
  Ok(())
}

fn main() {
  match solve() {
    Ok(()) => {},
    Err(msg) => println!("{}", msg),
  }
}
