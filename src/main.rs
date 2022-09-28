enum TokenKind {
  Num,
  Eof,
}

struct Token {
  kind: TokenKind,
  num: i32,
}

struct Parser {
  tokens_: Vec<Token>,
}

impl Parser {
  fn new() -> Self {
    Self {
      tokens_: Vec::new(),
    }
  }

  fn tokenize(&mut self, s: &str) -> Result<(), String> {
    if s.is_empty() {
      self.tokens_.push(Token {
        kind: TokenKind::Eof,
        num: 0,
      });
      return Ok(());
    }
    if s.starts_with(|c: char| c.is_whitespace()) {
      return self.skip_whitespace(&s);
    }
    if s.starts_with(|c: char| c.is_digit(10)) {
      return self.tokenize_num(&s);
    }
    Err("Unexpected character".to_string())
  }

  fn skip_whitespace(&mut self, s: &str) -> Result<(), String> {
    let pos = s.find(|c: char| !c.is_whitespace()).unwrap_or(s.len());
    self.tokenize(&s[pos..])
  }

  fn tokenize_num(&mut self, s: &str) -> Result<(), String> {
    let pos = s.find(|c: char| !c.is_digit(10)).unwrap_or(s.len());
    let res = s[..pos].parse::<i32>();
    if res.is_ok() {
      self.tokens_.push(Token {
        kind: TokenKind::Num,
        num: res.unwrap(),
      });
      self.tokenize(&s[pos..])
    } else {
      Err("Failed to convert".to_string())
    }
  }

  fn parse(&mut self) -> Result<i32, String> {
    match self.tokens_[0].kind {
      TokenKind::Num => Ok(self.tokens_[0].num),
      _ => Err("Unexpected Token".to_string()),
    }
  }
}

fn parse(s: &str) -> Result<i32, String> {
  let mut parser = Parser::new();
  let res = parser.tokenize(&s);
  if res.is_err() {
    Err(res.unwrap_err())
  } else {
    parser.parse()
  }
}

fn main() {
  assert!(parse("42") == Ok(42));
  assert!(parse("  123  ") == Ok(123));
}
