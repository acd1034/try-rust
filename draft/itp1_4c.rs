use std::io;

fn seq_apply<F, T, R>(mf: Option<F>, mx: Option<T>) -> Option<R>
where
  F: FnOnce(T) -> R,
{
  mf.and_then(|f| mx.and_then(|x| Some(f(x))))
}

fn lift_a2<F, T, U, R>(f: F, mx: Option<T>, my: Option<U>) -> Option<R>
where
  F: FnOnce(T, U) -> R,
{
  seq_apply(mx.map(|x| |y| f(x, y)), my)
}

fn read_expr() -> Option<i32> {
  let mut input = String::new();
  match io::stdin().read_line(&mut input) {
    Ok(_) => {
      let toks: Vec<_> = input.split(' ').collect();
      // println!("{:?}", toks);
      match toks[1] {
        "?" => None,
        op => {
          let left = toks[0].parse::<i32>().ok();
          let right = toks[2].trim().parse::<i32>().ok();
          // println!("{:?}", left);
          // println!("{:?}", right);
          let f = |x, y| match op {
            "+" => x + y,
            "-" => x - y,
            "*" => x * y,
            _ => x / y,
          };
          lift_a2(f, left, right)
        }
      }
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
