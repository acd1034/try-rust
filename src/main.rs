type Expected<T> = Result<T, &'static str>;

#[derive(Debug)]
enum Token {
  Eof,
  Punct(char),
  Num(i64),
}

fn tokenize<'a>(s: &'a str) -> Expected<(Token, &'a str)> {
  if s.is_empty() {
    Ok((Token::Eof, s))
  } else if s.starts_with(|c: char| c.is_whitespace()) {
    let pos = s.find(|c: char| !c.is_whitespace()).unwrap_or(s.len());
    tokenize(&s[pos..])
  } else if s.starts_with(|c: char| c.is_digit(10)) {
    let pos = s.find(|c: char| !c.is_digit(10)).unwrap_or(s.len());
    let num = s[..pos].parse::<i64>().map_err(|_| "Failed to read i64")?;
    Ok((Token::Num(num), &s[pos..]))
  } else if s.starts_with(|c: char| c.is_ascii_punctuation()) {
    Ok((Token::Punct(s.chars().next().unwrap()), &s[1..]))
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
}

impl<'a> Iterator for Tokenizer<'a> {
  type Item = Expected<Token>;
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

fn expect_num<F, T>(it: &mut Tokenizer, f: F) -> Expected<T>
where
  F: FnOnce(&mut Tokenizer, i64) -> Expected<T>,
{
  if let Token::Num(n) = it.next().unwrap()? {
    f(it, n)
  } else {
    Err("Unexpected token, expecting number")
  }
}

fn parse_expr_impl(it: &mut Tokenizer, n: i64) -> Expected<i64> {
  match it.next().unwrap()? {
    Token::Eof => Ok(n),
    Token::Punct('+') => expect_num(it, |it, m| parse_expr_impl(it, n + m)),
    Token::Punct('-') => expect_num(it, |it, m| parse_expr_impl(it, n - m)),
    _ => {
      return Err("Unexpected token");
    }
  }
}

fn parse_expr(it: &mut Tokenizer) -> Expected<i64> {
  expect_num(it, |it, n| parse_expr_impl(it, n))
}

fn parse(s: &str) -> Expected<i64> {
  let mut it = Tokenizer::new(s);
  parse_expr(&mut it)
}

#[test]
fn test1() {
  assert_eq!(parse("42"), Ok(42));
  assert_eq!(parse("  123  "), Ok(123));
  assert_eq!(parse("1 + 2 + 3 + 4"), Ok(10));
  assert_eq!(parse("1 + 2 - 3 + 4"), Ok(4));
}

fn main() {}
