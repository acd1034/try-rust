type Expected<T> = Result<T, &'static str>;

#[derive(Debug, PartialEq)]
enum Token<'a> {
  Eof,
  Punct(&'a str),
  Num(i64),
}

fn tokenize<'a>(s: &'a str) -> Expected<(Token, &'a str)> {
  if s.is_empty() {
    Ok((Token::Eof, s))
  } else if s.starts_with(|c: char| c.is_ascii_whitespace()) {
    let pos = s
      .find(|c: char| !c.is_ascii_whitespace())
      .unwrap_or(s.len());
    tokenize(&s[pos..])
  } else if s.starts_with(|c: char| c.is_digit(10)) {
    let pos = s.find(|c: char| !c.is_digit(10)).unwrap_or(s.len());
    let num = s[..pos]
      .parse::<i64>()
      .map_err(|_| "Failed to read integer")?;
    Ok((Token::Num(num), &s[pos..]))
  } else if s.starts_with(|c: char| c.is_ascii_punctuation()) {
    Ok((Token::Punct(&s[..1]), &s[1..]))
  } else {
    Err("Unexpected character")
  }
}

struct Tokenizer<'a> {
  input_: &'a str,
}

impl<'a> Tokenizer<'a> {
  fn new(input: &'a str) -> Tokenizer {
    Tokenizer { input_: input }
  }

  fn current(&mut self) -> Option<<Self as Iterator>::Item> {
    match tokenize(self.input_) {
      Ok((tok, _)) => Some(Ok(tok)),
      Err(msg) => Some(Err(msg)),
    }
  }
}

impl<'a> Iterator for Tokenizer<'a> {
  type Item = Expected<Token<'a>>;

  fn next(&mut self) -> Option<Self::Item> {
    match tokenize(self.input_) {
      // Ok((Token::Eof, s)) => {
      //   self.input_ = s;
      //   None
      // }
      Ok((tok, s)) => {
        self.input_ = s;
        Some(Ok(tok))
      }
      Err(msg) => Some(Err(msg)),
    }
  }
}

fn consume(it: &mut Tokenizer, op: &str) -> Expected<bool> {
  if it.current().unwrap()? == Token::Punct(op) {
    it.next();
    Ok(true)
  } else {
    Ok(false)
  }
}

fn expect_eof(it: &mut Tokenizer) -> Expected<()> {
  match it.current().unwrap()? {
    Token::Eof => Ok(()),
    _ => Err("Unexpected token, expecting eof"),
  }
}

fn expect_num(it: &mut Tokenizer) -> Expected<i64> {
  match it.current().unwrap()? {
    Token::Num(n) => {
      it.next();
      Ok(n)
    }
    _ => Err("Unexpected token, expecting number"),
  }
}

fn parse_term_impl(it: &mut Tokenizer, n: i64) -> Expected<i64> {
  if consume(it, "*")? {
    let m = expect_num(it)?;
    parse_term_impl(it, n * m)
  } else if consume(it, "/")? {
    let m = expect_num(it)?;
    parse_term_impl(it, n / m)
  } else {
    Ok(n)
  }
}

fn parse_term(it: &mut Tokenizer) -> Expected<i64> {
  let n = expect_num(it)?;
  parse_term_impl(it, n)
}

fn parse_expr_impl(it: &mut Tokenizer, n: i64) -> Expected<i64> {
  if consume(it, "+")? {
    let m = parse_term(it)?;
    parse_expr_impl(it, n + m)
  } else if consume(it, "-")? {
    let m = parse_term(it)?;
    parse_expr_impl(it, n - m)
  } else {
    Ok(n)
  }
}

fn parse_expr(it: &mut Tokenizer) -> Expected<i64> {
  let n = parse_term(it)?;
  let n = parse_expr_impl(it, n)?;
  expect_eof(it)?;
  Ok(n)
}

fn parse(s: &str) -> Expected<i64> {
  let mut it = Tokenizer::new(s);
  parse_expr(&mut it)
}

#[test]
fn test1() {
  // num
  assert_eq!(parse("42"), Ok(42));
  assert_eq!(parse("  123  "), Ok(123));

  // expr
  assert_eq!(parse("1 + 2 + 3 + 4"), Ok(10));
  assert_eq!(parse("1 + 2 - 3 + 4"), Ok(4));
  assert_eq!(parse("_ + 2").ok(), None);
  assert_eq!(parse("1 _ 2").ok(), None);
  assert_eq!(parse("1 + _").ok(), None);

  // term
  assert_eq!(parse("1 * 2 * 3 * 4"), Ok(24));
  assert_eq!(parse("3 * 4 / 6 * 2"), Ok(4));
  assert_eq!(parse("1 * 2 + 3 * 4 + 5 * 6"), Ok(44));
  assert_eq!(parse("1 * 2 - 6 / 3 + 4 * 5"), Ok(20));
  assert_eq!(parse("_ * 2").ok(), None);
  assert_eq!(parse("1 _ 2").ok(), None);
  assert_eq!(parse("1 * _").ok(), None);
}

fn main() {
  match parse("42") {
    Ok(n) => println!("{}", n),
    Err(msg) => println!("{}", msg),
  }
}
