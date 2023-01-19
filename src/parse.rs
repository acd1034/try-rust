use crate::tokenize::{Expected, Token, Tokenizer};

#[derive(Debug)]
pub enum Function {
  Function(String, Vec<String>, Vec<Stmt>),
  Prototype(String, Vec<String>),
}

#[derive(Debug)]
pub enum Stmt {
  IfElse(AST, Box<Stmt>, Option<Box<Stmt>>),
  For(Option<AST>, Option<AST>, Option<AST>, Box<Stmt>),
  Return(AST),
  Block(Vec<Stmt>),
  Expr(AST),
}

#[derive(Debug)]
pub enum AST {
  Assign(Box<AST>, Box<AST>),
  Eq(Box<AST>, Box<AST>),
  Ne(Box<AST>, Box<AST>),
  Lt(Box<AST>, Box<AST>),
  Le(Box<AST>, Box<AST>),
  Add(Box<AST>, Box<AST>),
  Sub(Box<AST>, Box<AST>),
  Mul(Box<AST>, Box<AST>),
  Div(Box<AST>, Box<AST>),
  Call(String),
  Ident(String),
  Num(u64),
}

fn consume_eof(it: &mut Tokenizer) -> Expected<bool> {
  if it.current().unwrap()? == Token::Eof {
    Ok(true)
  } else {
    Ok(false)
  }
}

fn consume_keyword(it: &mut Tokenizer, keyword: &str) -> Expected<bool> {
  if it.current().unwrap()? == Token::Keyword(keyword) {
    it.next();
    Ok(true)
  } else {
    Ok(false)
  }
}

fn consume(it: &mut Tokenizer, op: &str) -> Expected<bool> {
  if it.current().unwrap()? == Token::Punct(op) {
    it.next();
    Ok(true)
  } else {
    Ok(false)
  }
}

fn expect_ident(it: &mut Tokenizer) -> Expected<String> {
  match it.current().unwrap()? {
    Token::Ident(name) => {
      it.next();
      Ok(name.to_string())
    }
    _ => Err("unexpected token, expecting identifier"),
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

/* program    = function* eof
 * function   = ident params "{" statement* "}"
 *            | ident params ";"
 * params     = "(" (ident ("," ident)*)? ")"
 * statement  = "if" "(" expr ")" statement ("else" statement)?
 *            | "for" "(" expr? ";" expr? ";" expr? ")" statement
 *            | "return" expr ";"
 *            | "{" statement* "}"
 *            | ";"
 *            | expr ";"
 * expr       = assign
 * assign     = equality ("=" assign)?
 * equality   = relational ("==" relational | "!=" relational)*
 * relational = add ("<" add | "<=" add | ">" add | ">=" add)*
 * add        = mul ("+" mul | "-" mul)*
 * mul        = unary ("*" unary | "/" unary)*
 * unary      = ("+" | "-")? unary | primary
 * primary    = "(" expr ")" | ident ("(" ")")? | num
 */

// program    = function* eof
pub fn parse(mut it: Tokenizer) -> Expected<Vec<Function>> {
  let mut functions = Vec::new();
  while !consume_eof(&mut it)? {
    functions.push(parse_function(&mut it)?);
  }
  Ok(functions)
}

// function   = ident params "{" statement* "}"
//            | ident params ";"
fn parse_function(it: &mut Tokenizer) -> Expected<Function> {
  let name = expect_ident(it)?;
  let params = parse_params(it)?;
  if consume(it, "{")? {
    let mut body = Vec::new();
    while !consume(it, "}")? {
      body.push(parse_statement(it)?);
    }
    Ok(Function::Function(name, params, body))
  } else {
    expect(it, ";")?;
    Ok(Function::Prototype(name, params))
  }
}

// params     = "(" (ident ("," ident)*)? ")"
fn parse_params(it: &mut Tokenizer) -> Expected<Vec<String>> {
  expect(it, "(")?;
  let mut params = Vec::new();
  if consume(it, ")")? {
    Ok(params)
  } else {
    params.push(expect_ident(it)?);
    while !consume(it, ")")? {
      expect(it, ",")?;
      params.push(expect_ident(it)?);
    }
    Ok(params)
  }
}

// statement  = "if" "(" expr ")" statement ("else" statement)?
//            | "for" "(" expr? ";" expr? ";" expr? ")" statement
//            | "return" expr ";"
//            | "{" statement* "}"
//            | ";"
//            | expr ";"
fn parse_statement(it: &mut Tokenizer) -> Expected<Stmt> {
  if consume_keyword(it, "if")? {
    expect(it, "(")?;
    let cond = parse_expr(it)?;
    expect(it, ")")?;
    let then_stmt = Box::new(parse_statement(it)?);
    let else_stmt = if consume_keyword(it, "else")? {
      Some(Box::new(parse_statement(it)?))
    } else {
      None
    };
    Ok(Stmt::IfElse(cond, then_stmt, else_stmt))
  } else if consume_keyword(it, "for")? {
    expect(it, "(")?;
    let n1 = parse_expr(it).ok();
    expect(it, ";")?;
    let n2 = parse_expr(it).ok();
    expect(it, ";")?;
    let n3 = parse_expr(it).ok();
    expect(it, ")")?;
    let stmt = Box::new(parse_statement(it)?);
    Ok(Stmt::For(n1, n2, n3, stmt))
  } else if consume_keyword(it, "return")? {
    let n = parse_expr(it)?;
    expect(it, ";")?;
    Ok(Stmt::Return(n))
  } else if consume(it, "{")? {
    let mut stmts = Vec::new();
    while !consume(it, "}")? {
      stmts.push(parse_statement(it)?);
    }
    Ok(Stmt::Block(stmts))
  } else if consume(it, ";")? {
    Ok(Stmt::Block(Vec::new()))
  } else {
    let n = parse_expr(it)?;
    expect(it, ";")?;
    Ok(Stmt::Expr(n))
  }
}

// expr       = assign
fn parse_expr(it: &mut Tokenizer) -> Expected<AST> {
  parse_assign(it)
}

// assign     = equality ("=" assign)?
fn parse_assign(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_equality(it)?;
  if consume(it, "=")? {
    let m = parse_assign(it)?;
    Ok(AST::Assign(Box::new(n), Box::new(m)))
  } else {
    Ok(n)
  }
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

// relational = add ("<" add | "<=" add | ">" add | ">=" add)*
fn parse_relational(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_add(it)?;
  parse_relational_impl(it, n)
}

fn parse_relational_impl(it: &mut Tokenizer, n: AST) -> Expected<AST> {
  if consume(it, "<")? {
    let m = parse_add(it)?;
    parse_relational_impl(it, AST::Lt(Box::new(n), Box::new(m)))
  } else if consume(it, "<=")? {
    let m = parse_add(it)?;
    parse_relational_impl(it, AST::Le(Box::new(n), Box::new(m)))
  } else if consume(it, ">")? {
    let m = parse_add(it)?;
    parse_relational_impl(it, AST::Lt(Box::new(m), Box::new(n)))
  } else if consume(it, ">=")? {
    let m = parse_add(it)?;
    parse_relational_impl(it, AST::Le(Box::new(m), Box::new(n)))
  } else {
    Ok(n)
  }
}

// add        = mul ("+" mul | "-" mul)*
fn parse_add(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_mul(it)?;
  parse_add_impl(it, n)
}

fn parse_add_impl(it: &mut Tokenizer, n: AST) -> Expected<AST> {
  if consume(it, "+")? {
    let m = parse_mul(it)?;
    parse_add_impl(it, AST::Add(Box::new(n), Box::new(m)))
  } else if consume(it, "-")? {
    let m = parse_mul(it)?;
    parse_add_impl(it, AST::Sub(Box::new(n), Box::new(m)))
  } else {
    Ok(n)
  }
}

// mul        = unary ("*" unary | "/" unary)*
fn parse_mul(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_unary(it)?;
  parse_mul_impl(it, n)
}

fn parse_mul_impl(it: &mut Tokenizer, n: AST) -> Expected<AST> {
  if consume(it, "*")? {
    let m = parse_unary(it)?;
    parse_mul_impl(it, AST::Mul(Box::new(n), Box::new(m)))
  } else if consume(it, "/")? {
    let m = parse_unary(it)?;
    parse_mul_impl(it, AST::Div(Box::new(n), Box::new(m)))
  } else {
    Ok(n)
  }
}

// unary      = ("+" | "-")? unary | primary
fn parse_unary(it: &mut Tokenizer) -> Expected<AST> {
  if consume(it, "+")? {
    parse_unary(it)
  } else if consume(it, "-")? {
    let n = AST::Num(0);
    let m = parse_unary(it)?;
    Ok(AST::Sub(Box::new(n), Box::new(m)))
  } else {
    parse_primary(it)
  }
}

// primary    = "(" expr ")" | ident ("(" ")")? | num
fn parse_primary(it: &mut Tokenizer) -> Expected<AST> {
  if consume(it, "(")? {
    let n = parse_expr(it)?;
    expect(it, ")")?;
    Ok(n)
  } else if let Token::Ident(name) = it.current().unwrap()? {
    it.next();
    if consume(it, "(")? {
      expect(it, ")")?;
      Ok(AST::Call(name.to_string()))
    } else {
      Ok(AST::Ident(name.to_string()))
    }
  } else if let Token::Num(n) = it.current().unwrap()? {
    it.next();
    Ok(AST::Num(n))
  } else {
    Err("unexpected token, expecting `(`, identifier or number")
  }
}
