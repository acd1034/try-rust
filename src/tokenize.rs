#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
  Eof,
  Keyword(&'a str),
  Ident(String),
  Num(u64),
  Punct(&'a str),
  Invalid(&'a str),
}

fn tokenize<'a>(s: &'a str) -> (Token, &'a str) {
  static KEYWORDS: [&str; 8] = [
    "return", "if", "else", "for", "while", "break", "continue", "int",
  ];
  static TWO_CHAR_OPS: [&str; 10] = ["==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "++", "--"];

  if s.is_empty() {
    (Token::Eof, s)
  } else if s.starts_with(|c: char| c.is_ascii_whitespace()) {
    let pos = s
      .find(|c: char| !c.is_ascii_whitespace())
      .unwrap_or(s.len());
    tokenize(&s[pos..])
  } else if s.starts_with(|c: char| c == '_' || c.is_ascii_alphabetic()) {
    let pos = s
      .find(|c: char| c != '_' && !c.is_ascii_alphabetic() && !c.is_ascii_digit())
      .unwrap_or(s.len());
    if KEYWORDS.contains(&&s[..pos]) {
      (Token::Keyword(&s[..pos]), &s[pos..])
    } else {
      (Token::Ident(s[..pos].to_string()), &s[pos..])
    }
  } else if s.starts_with(|c: char| c.is_ascii_digit()) {
    let pos = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let num: u64 = s[..pos].parse().unwrap();
    (Token::Num(num), &s[pos..])
  } else if s.starts_with(|c: char| c.is_ascii_punctuation()) {
    if s.len() < 2 {
      (Token::Punct(&s[..1]), &s[1..])
    } else if TWO_CHAR_OPS.contains(&&s[..2]) {
      (Token::Punct(&s[..2]), &s[2..])
    } else {
      (Token::Punct(&s[..1]), &s[1..])
    }
  } else {
    (Token::Invalid(&s[..1]), &s[1..])
  }
}

pub struct Tokenizer<'a> {
  item: Token<'a>,
  input: &'a str,
}

impl<'a> Tokenizer<'a> {
  pub fn new(input: &'a str) -> Tokenizer {
    let (tok, s) = tokenize(input);
    Tokenizer {
      item: tok,
      input: s,
    }
  }

  pub fn advance(&mut self) -> () {
    let (tok, s) = tokenize(self.input);
    self.item = tok;
    self.input = s;
  }

  pub fn current(&mut self) -> Token<'a> {
    self.item.clone()
  }
}
