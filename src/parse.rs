use crate::tokenize::{Expected, Token, Tokenizer};

#[derive(Debug, PartialEq)]
pub enum AST {
  Eq(Box<AST>, Box<AST>),
  Ne(Box<AST>, Box<AST>),
  Lt(Box<AST>, Box<AST>),
  Le(Box<AST>, Box<AST>),
  Add(Box<AST>, Box<AST>),
  Sub(Box<AST>, Box<AST>),
  Mul(Box<AST>, Box<AST>),
  Div(Box<AST>, Box<AST>),
  Num(u64),
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
 * program    = equality eof
 * equality   = relational ("==" relational | "!=" relational)*
 * relational = expr ("<" expr | "<=" expr | ">" expr | ">=" expr)*
 * expr       = term ("+" term | "-" term)*
 * term       = unary ("*" unary | "/" unary)*
 * unary      = ("+" | "-")? primary
 * primary    = "(" expr ")" | num
 */

// program    = equality eof
pub fn parse(mut it: Tokenizer) -> Expected<AST> {
  let n = parse_equality(&mut it)?;
  expect_eof(&mut it)?;
  Ok(n)
}

// equality   = relational ("==" relational | "!=" relational)*
fn parse_equality(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_relational(it)?;
  parse_equality_impl(it, n)
}

fn parse_equality_impl(it: &mut Tokenizer, n: AST) -> Expected<AST> {
  if consume(it, "==")? {
    let m = parse_relational(it)?;
    parse_equality_impl(it, AST::Eq(Box::new(n), Box::new(m)))
  } else if consume(it, "!=")? {
    let m = parse_relational(it)?;
    parse_equality_impl(it, AST::Ne(Box::new(n), Box::new(m)))
  } else {
    Ok(n)
  }
}

// relational = expr ("<" expr | "<=" expr | ">" expr | ">=" expr)*
fn parse_relational(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_expr(it)?;
  parse_relational_impl(it, n)
}

fn parse_relational_impl(it: &mut Tokenizer, n: AST) -> Expected<AST> {
  if consume(it, "<")? {
    let m = parse_expr(it)?;
    parse_relational_impl(it, AST::Lt(Box::new(n), Box::new(m)))
  } else if consume(it, "<=")? {
    let m = parse_expr(it)?;
    parse_relational_impl(it, AST::Le(Box::new(n), Box::new(m)))
  } else if consume(it, ">")? {
    let m = parse_expr(it)?;
    parse_relational_impl(it, AST::Lt(Box::new(m), Box::new(n)))
  } else if consume(it, ">=")? {
    let m = parse_expr(it)?;
    parse_relational_impl(it, AST::Le(Box::new(m), Box::new(n)))
  } else {
    Ok(n)
  }
}

// expr       = term ("+" term | "-" term)*
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

// term       = unary ("*" unary | "/" unary)*
fn parse_term(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_unary(it)?;
  parse_term_impl(it, n)
}

fn parse_term_impl(it: &mut Tokenizer, n: AST) -> Expected<AST> {
  if consume(it, "*")? {
    let m = parse_unary(it)?;
    parse_term_impl(it, AST::Mul(Box::new(n), Box::new(m)))
  } else if consume(it, "/")? {
    let m = parse_unary(it)?;
    parse_term_impl(it, AST::Div(Box::new(n), Box::new(m)))
  } else {
    Ok(n)
  }
}

// unary      = ("+" | "-")? primary
fn parse_unary(it: &mut Tokenizer) -> Expected<AST> {
  if consume(it, "+")? {
    parse_primary(it)
  } else if consume(it, "-")? {
    let n = AST::Num(0);
    let m = parse_primary(it)?;
    Ok(AST::Sub(Box::new(n), Box::new(m)))
  } else {
    parse_primary(it)
  }
}

// primary    = "(" expr ")" | num
fn parse_primary(it: &mut Tokenizer) -> Expected<AST> {
  if consume(it, "(")? {
    let n = parse_expr(it)?;
    expect(it, ")")?;
    Ok(n)
  } else {
    expect_num(it)
  }
}
