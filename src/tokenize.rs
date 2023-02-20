pub type Expected<T> = Result<T, &'static str>;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
  Eof,
  Keyword(&'a str),
  Ident(&'a str),
  Num(u64),
  Punct(&'a str),
}

fn tokenize<'a>(s: &'a str) -> Expected<(Token, &'a str)> {
  static KEYWORDS: [&str; 7] = ["return", "if", "else", "for", "while", "int", "continue"];
  static TWO_CHAR_OPS: [&str; 10] = ["==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "++", "--"];
  if s.is_empty() {
    Ok((Token::Eof, s))
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
      Ok((Token::Keyword(&s[..pos]), &s[pos..]))
    } else {
      Ok((Token::Ident(&s[..pos]), &s[pos..]))
    }
  } else if s.starts_with(|c: char| c.is_ascii_digit()) {
    let pos = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let num: u64 = s[..pos].parse().map_err(|_| "failed to read integer")?;
    Ok((Token::Num(num), &s[pos..]))
  } else if s.starts_with(|c: char| c.is_ascii_punctuation()) {
    if s.len() < 2 {
      return Ok((Token::Punct(&s[..1]), &s[1..]));
    }
    if TWO_CHAR_OPS.contains(&&s[..2]) {
      Ok((Token::Punct(&s[..2]), &s[2..]))
    } else {
      Ok((Token::Punct(&s[..1]), &s[1..]))
    }
  } else {
    Err("unexpected character")
  }
}

pub struct Tokenizer<'a> {
  item: Expected<Token<'a>>,
  input: &'a str,
}

impl<'a> Tokenizer<'a> {
  pub fn new(input: &'a str) -> Tokenizer {
    match tokenize(input) {
      Ok((tok, s)) => Tokenizer {
        item: Ok(tok),
        input: s,
      },
      Err(msg) => Tokenizer {
        item: Err(msg),
        input,
      },
    }
  }

  pub fn advance(&mut self) -> () {
    match tokenize(self.input) {
      Ok((tok, s)) => {
        self.item = Ok(tok);
        self.input = s;
      }
      Err(msg) => {
        self.item = Err(msg);
      }
    }
  }

  pub fn current(&mut self) -> Expected<Token<'a>> {
    self.item.clone()
  }
}
