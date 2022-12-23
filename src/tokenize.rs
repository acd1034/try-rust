pub type Expected<T> = Result<T, &'static str>;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
  Eof,
  Ident(&'a str),
  Num(u64),
  Punct(&'a str),
}

fn tokenize<'a>(s: &'a str) -> Expected<(Token, &'a str)> {
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
    Ok((Token::Ident(&s[..pos]), &s[pos..]))
  } else if s.starts_with(|c: char| c.is_ascii_digit()) {
    let pos = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let num = s[..pos]
      .parse::<u64>()
      .map_err(|_| "Failed to read integer")?;
    Ok((Token::Num(num), &s[pos..]))
  } else if s.starts_with(|c: char| c.is_ascii_punctuation()) {
    if s.len() < 2 {
      Ok((Token::Punct(&s[..1]), &s[1..]))
    } else {
      match &s[..2] {
        "==" | "!=" | "<=" | ">=" => Ok((Token::Punct(&s[..2]), &s[2..])),
        _ => Ok((Token::Punct(&s[..1]), &s[1..])),
      }
    }
  } else {
    Err("unexpected character")
  }
}

pub struct Tokenizer<'a> {
  input_: &'a str,
}

impl<'a> Tokenizer<'a> {
  pub fn new(input: &'a str) -> Tokenizer {
    Tokenizer { input_: input }
  }

  pub fn current(&mut self) -> Option<<Self as Iterator>::Item> {
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
