use crate::{common::Expected, err};

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
  Eof,
  Keyword(&'a str),
  Ident(String),
  Num(u64),
  Str(String),
  Punct(&'a str),
}

fn tokenize<'a>(s: &'a str) -> (Expected<Token<'a>>, &'a str) {
  static KEYWORDS: [&str; 9] = [
    "return", "if", "else", "for", "while", "break", "continue", "int", "char",
  ];
  static TWO_CHAR_OPS: [&str; 10] = ["==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "++", "--"];

  if s.is_empty() {
    (Ok(Token::Eof), s)
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
      (Ok(Token::Keyword(&s[..pos])), &s[pos..])
    } else {
      (Ok(Token::Ident(s[..pos].to_string())), &s[pos..])
    }
  } else if s.starts_with(|c: char| c.is_ascii_digit()) {
    let pos = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let tok = if let Ok(num) = s[..pos].parse() {
      Ok(Token::Num(num))
    } else {
      err!("failed to read integer")
    };
    (tok, &s[pos..])
  } else if s.starts_with('"') {
    if let Some(pos) = s[1..].find('"') {
      (Ok(Token::Str(s[1..pos + 1].to_string())), &s[pos + 2..])
    } else {
      (err!("missing terminating `\"` character"), &s[s.len()..])
    }
  } else if s.starts_with(|c: char| c.is_ascii_punctuation()) {
    if s.len() < 2 {
      (Ok(Token::Punct(&s[..1])), &s[1..])
    } else if TWO_CHAR_OPS.contains(&&s[..2]) {
      (Ok(Token::Punct(&s[..2])), &s[2..])
    } else {
      (Ok(Token::Punct(&s[..1])), &s[1..])
    }
  } else {
    (err!("unexpected character"), &s[1..])
  }
}

#[derive(Clone)]
pub struct Tokenizer<'a> {
  item: Expected<Token<'a>>,
  input: &'a str,
}

impl<'a> Tokenizer<'a> {
  pub fn new(input: &'a str) -> Tokenizer {
    let (item, input) = tokenize(input);
    Tokenizer { item, input }
  }

  pub fn advance(&mut self) -> () {
    let (item, input) = tokenize(self.input);
    self.item = item;
    self.input = input;
  }

  pub fn current(&mut self) -> Expected<Token<'a>> {
    self.item.clone()
  }
}
