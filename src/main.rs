#[derive(PartialEq)]
enum TokenKind {
  Num,
  Punct,
  Eof,
}

struct Token {
  kind: TokenKind,
  c: char,
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
        c: '\0',
        num: 0,
      });
      Ok(())
    } else if s.starts_with(|c: char| c.is_whitespace()) {
      self.skip_whitespace(&s)
    } else if s.starts_with(|c: char| c.is_digit(10)) {
      self.tokenize_num(&s)
    } else if s.starts_with(|c: char| c.is_ascii_punctuation()) {
      self.tokenize_punct(&s)
    } else {
      Err("Unexpected character".to_string())
    }
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
        c: '\0',
        num: res.unwrap(),
      });
      self.tokenize(&s[pos..])
    } else {
      Err("Failed to convert".to_string())
    }
  }

  fn tokenize_punct(&mut self, s: &str) -> Result<(), String> {
    self.tokens_.push(Token {
      kind: TokenKind::Punct,
      c: s.chars().next().unwrap(),
      num: 0,
    });
    self.tokenize(&s[1..])
  }

  fn parse(&self) -> Result<i32, String> {
    let tokens = &self.tokens_[..];
    if tokens[0].kind == TokenKind::Num {
      let val = tokens[0].num;
      self.parse_expr(&tokens[1..], val)
    } else {
      Err("Unexpected Token, expecting number".to_string())
    }
  }

  fn parse_expr(&self, tokens: &[Token], val: i32) -> Result<i32, String> {
    if tokens[0].kind == TokenKind::Eof {
      Ok(val)
    } else if tokens[0].kind == TokenKind::Punct {
      match tokens[0].c {
        '+' => {
          if tokens[1].kind == TokenKind::Num {
            self.parse_expr(&tokens[2..], val + tokens[1].num)
          } else {
            Err("Unexpected Token, expecting number".to_string())
          }
        }
        '-' => {
          if tokens[1].kind == TokenKind::Num {
            self.parse_expr(&tokens[2..], val - tokens[1].num)
          } else {
            Err("Unexpected Token, expecting number".to_string())
          }
        }
        _ => Err("Unexpected punctuator, expecting '+' or '-'".to_string()),
      }
    } else {
      Err("Unexpected Token".to_string())
    }
  }
}

fn parse(s: &str) -> Result<i32, String> {
  let mut parser = Parser::new();
  let res = parser.tokenize(&s);
  if res.is_ok() {
    parser.parse()
  } else {
    Err(res.unwrap_err())
  }
}

fn main() {
  assert!(parse("42") == Ok(42));
  assert!(parse("  123  ") == Ok(123));
  assert!(parse("1 + 2 + 3 + 4") == Ok(10));
  assert!(parse("1 + 2 - 3 + 4") == Ok(4));
}
