use crate::tokenize::*;

#[derive(Debug, PartialEq)]
pub enum AST {
  Add(Box<AST>, Box<AST>),
  Sub(Box<AST>, Box<AST>),
  Mul(Box<AST>, Box<AST>),
  Div(Box<AST>, Box<AST>),
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
    _ => Err("unexpected token, expecting eof"),
  }
}

fn expect_num(it: &mut Tokenizer) -> Expected<AST> {
  match it.current().unwrap()? {
    Token::Num(n) => {
      it.next();
      Ok(AST::Num(n))
    }
    _ => Err("unexpected token, expecting number"),
  }
}

fn expect(it: &mut Tokenizer, op: &str) -> Expected<()> {
  if it.current().unwrap()? == Token::Punct(op) {
    it.next();
    Ok(())
  } else {
    Err("unexpected token, expecting punctuator")
  }
}

/**
 * program = expr eof
 * expr    = term ("+" term | "-" term)*
 * term    = primary ("*" primary | "/" primary)*
 * primary = "(" expr ")" | num
 */

// program = expr eof
pub fn parse(mut it: Tokenizer) -> Expected<AST> {
  let n = parse_expr(&mut it)?;
  expect_eof(&mut it)?;
  Ok(n)
}

// expr    = term ("+" term | "-" term)*
fn parse_expr(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_term(it)?;
  parse_expr_impl(it, n)
}

fn parse_expr_impl(it: &mut Tokenizer, n: AST) -> Expected<AST> {
  if consume(it, "+")? {
    let m = parse_term(it)?;
    parse_expr_impl(it, AST::Add(Box::new(n), Box::new(m)))
  } else if consume(it, "-")? {
    let m = parse_term(it)?;
    parse_expr_impl(it, AST::Sub(Box::new(n), Box::new(m)))
  } else {
    Ok(n)
  }
}

// term    = primary ("*" primary | "/" primary)*
fn parse_term(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_primary(it)?;
  parse_term_impl(it, n)
}

fn parse_term_impl(it: &mut Tokenizer, n: AST) -> Expected<AST> {
  if consume(it, "*")? {
    let m = parse_primary(it)?;
    parse_term_impl(it, AST::Mul(Box::new(n), Box::new(m)))
  } else if consume(it, "/")? {
    let m = parse_primary(it)?;
    parse_term_impl(it, AST::Div(Box::new(n), Box::new(m)))
  } else {
    Ok(n)
  }
}

// primary = "(" expr ")" | num
fn parse_primary(it: &mut Tokenizer) -> Expected<AST> {
  if consume(it, "(")? {
    let n = parse_expr(it)?;
    expect(it, ")")?;
    Ok(n)
  } else {
    expect_num(it)
  }
}
