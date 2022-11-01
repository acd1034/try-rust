use std::io;

fn read_expr() -> Option<i32> {
  let mut input = String::new();
  match io::stdin().read_line(&mut input) {
    Ok(_) => {
      let toks: Vec<_> = input.split(' ').collect();
      // println!("{:?}", toks);
      toks[0].parse::<i32>().ok().and_then(|x| {
        toks[2]
          .trim()
          .parse::<i32>()
          .ok()
          .and_then(|y| match toks[1] {
            "+" => Some(x + y),
            "-" => Some(x - y),
            "*" => Some(x * y),
            "/" => Some(x / y),
            _ => None,
          })
      })
    }
    Err(_) => None,
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
