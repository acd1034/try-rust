type Expected<T> = Result<T, &'static str>;

/******************************************************************************
 * Tokenizer
 ******************************************************************************/

#[derive(Debug, PartialEq)]
enum Token<'a> {
  Eof,
  Num(i64),
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

/******************************************************************************
 * Parser
 ******************************************************************************/

#[derive(Debug, PartialEq)]
enum AST {
  Num(i64),
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

fn expect_num(it: &mut Tokenizer) -> Expected<AST> {
  match it.current().unwrap()? {
    Token::Num(n) => {
      it.next();
      Ok(AST::Num(n))
    }
    _ => Err("Unexpected token, expecting number"),
  }
}

fn expect(it: &mut Tokenizer, op: &str) -> Expected<()> {
  if it.current().unwrap()? == Token::Punct(op) {
    it.next();
    Ok(())
  } else {
    Err("Unexpected token, expecting punctuator")
  }
}

/**
 * program = expr eof
 * expr    = term ("+" term | "-" term)*
 * → expr  = expr | term
 * term    = primary ("*" primary | "/" primary)*
 * → term  = term | primary
 * primary = num | "(" expr ")"
 */

// program = expr eof
fn parse(mut it: Tokenizer) -> Expected<AST> {
  let n = parse_primary(&mut it)?;
  expect_eof(&mut it)?;
  Ok(n)
}

// // expr    = term ("+" term | "-" term)*
// // → expr  = expr | term
// fn parse_expr(it: &mut Tokenizer) -> Expected<i64> {
//   let n = parse_term(it)?;
//   parse_expr_impl(it, n)
// }

// fn parse_expr_impl(it: &mut Tokenizer, n: i64) -> Expected<i64> {
//   if consume(it, "+")? {
//     let m = parse_term(it)?;
//     parse_expr_impl(it, n + m)
//   } else if consume(it, "-")? {
//     let m = parse_term(it)?;
//     parse_expr_impl(it, n - m)
//   } else {
//     Ok(n)
//   }
// }

// // term    = primary ("*" primary | "/" primary)*
// // → term  = term | primary
// fn parse_term(it: &mut Tokenizer) -> Expected<i64> {
//   let n = parse_primary(it)?;
//   parse_term_impl(it, n)
// }

// fn parse_term_impl(it: &mut Tokenizer, n: i64) -> Expected<i64> {
//   if consume(it, "*")? {
//     let m = parse_primary(it)?;
//     parse_term_impl(it, n * m)
//   } else if consume(it, "/")? {
//     let m = parse_primary(it)?;
//     parse_term_impl(it, n / m)
//   } else {
//     Ok(n)
//   }
// }

// primary = num | "(" expr ")"
fn parse_primary(it: &mut Tokenizer) -> Expected<AST> {
  // if consume(it, "(")? {
  //   let n = parse_expr(it)?;
  //   expect(it, ")")?;
  //   Ok(n)
  // } else {
  //   expect_num(it)
  // }
  expect_num(it)
}

/******************************************************************************
 * Codegen
 ******************************************************************************/

fn codegen(ast: AST) -> String {
  format!("{:?}", ast)
}

/******************************************************************************
 * Compiler
 ******************************************************************************/

fn compile(s: &str) -> Expected<String> {
  let it = Tokenizer::new(s);
  let ast = parse(it)?;
  Ok(codegen(ast))
}

fn test(s: &str) {
  match compile(s) {
    Ok(n) => println!("{}", n),
    Err(msg) => println!("error: {}", msg),
  }
}

fn main() {
  test("42");
  test("42_");
  // match compile("42") {
  //   Ok(n) => println!("{}", n),
  //   Err(msg) => println!("error: {}", msg),
  // }
}
