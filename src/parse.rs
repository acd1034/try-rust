use crate::tokenize::{Token, Tokenizer};
use crate::ty::Type;
use crate::{common::Expected, err};

#[derive(Debug)]
pub enum Fun {
  FunDecl(Type, String, Vec<Type>),
  FunDef(Type, String, Vec<Type>, Vec<String>, Vec<Stmt>),
}

#[derive(Debug)]
pub enum Stmt {
  VarDef(Type, String, Option<AST>),
  IfElse(AST, Box<Stmt>, Option<Box<Stmt>>),
  For(Option<AST>, Option<AST>, Option<AST>, Box<Stmt>),
  Break,
  Cont,
  Return(AST),
  Block(Vec<Stmt>),
  Expr(AST),
}

#[derive(Clone, Debug)]
pub enum AST {
  Ternary(Box<AST>, Box<AST>, Box<AST>),
  Assign(Box<AST>, Box<AST>),
  Eq(Box<AST>, Box<AST>),
  Ne(Box<AST>, Box<AST>),
  Lt(Box<AST>, Box<AST>),
  Le(Box<AST>, Box<AST>),
  Add(Box<AST>, Box<AST>),
  Sub(Box<AST>, Box<AST>),
  Mul(Box<AST>, Box<AST>),
  Div(Box<AST>, Box<AST>),
  Addr(Box<AST>),
  Deref(Box<AST>),
  Call(String, Vec<AST>),
  Ident(String),
  Num(u64),
}

fn consume_eof(it: &mut Tokenizer) -> Expected<bool> {
  if it.current()? == Token::Eof {
    Ok(true)
  } else {
    Ok(false)
  }
}

fn consume_keyword(it: &mut Tokenizer, keyword: &str) -> Expected<bool> {
  if it.current()? == Token::Keyword(keyword) {
    it.advance();
    Ok(true)
  } else {
    Ok(false)
  }
}

fn consume_ident(it: &mut Tokenizer) -> Expected<Option<String>> {
  if let Token::Ident(name) = it.current()? {
    it.advance();
    Ok(Some(name.to_string()))
  } else {
    Ok(None)
  }
}

fn consume_num(it: &mut Tokenizer) -> Expected<Option<u64>> {
  if let Token::Num(n) = it.current()? {
    it.advance();
    Ok(Some(n))
  } else {
    Ok(None)
  }
}

fn consume(it: &mut Tokenizer, op: &str) -> Expected<bool> {
  if it.current()? == Token::Punct(op) {
    it.advance();
    Ok(true)
  } else {
    Ok(false)
  }
}

fn expect_ident(it: &mut Tokenizer) -> Expected<String> {
  if let Token::Ident(name) = it.current()? {
    it.advance();
    Ok(name.to_string())
  } else {
    err!("unexpected token, expecting identifier")
  }
}

fn expect_num(it: &mut Tokenizer) -> Expected<u64> {
  if let Token::Num(n) = it.current()? {
    it.advance();
    Ok(n)
  } else {
    err!("unexpected token, expecting number")
  }
}

fn expect(it: &mut Tokenizer, op: &str) -> Expected<()> {
  if it.current()? == Token::Punct(op) {
    it.advance();
    Ok(())
  } else {
    err!("unexpected token, expecting punctuator")
  }
}

/* program     = fun* eof
 * fun         = declspec declarator fun_params ";"
 *             | declspec declarator fun_params "{" stmt* "}"
 * declspec    = "int"
 * declarator  = "*"* ident ("[" num "]")?
 * fun_params  = "(" param (("," param)*)? ")"
 * param       = declspec declarator
 *
 * stmt        = declspec declarator ("=" expr)? ";"
 *             | "if" "(" expr ")" stmt ("else" stmt)?
 *             | "for" "(" expr? ";" expr? ";" expr? ")" stmt
 *             | "while" "(" expr ")" stmt
 *             | "break" ";"
 *             | "continue" ";"
 *             | "return" expr ";"
 *             | "{" stmt* "}"
 *             | ";"
 *             | expr ";"
 *
 * expr        = ternary
 * ternary     = assign ("?" expr ":" ternary)?
 * assign      = equality ("=" assign | "+=" assign | "-=" assign | "*=" assign | "/=" assign)?
 * equality    = relational ("==" relational | "!=" relational)*
 * relational  = add ("<" add | "<=" add | ">" add | ">=" add)*
 * add         = mul ("+" mul | "-" mul)*
 * mul         = unary ("*" unary | "/" unary)*
 * unary       = ("+" | "-" | "&" | "*" | "++" | "--") unary
 *             | postfix
 * postfix     = primary ("[" expr "]" | "++" | "--")?
 * primary     = "(" expr ")"
 *             | ident "(" fun_args
 *             | ident
 *             | num
 * fun_args    = (expr ("," expr)*)? ")"
 */

// program     = fun* eof
pub fn parse(mut it: Tokenizer) -> Expected<Vec<Fun>> {
  let mut funs = Vec::new();
  while !consume_eof(&mut it)? {
    funs.push(parse_fun(&mut it)?);
  }
  Ok(funs)
}

// fun         = declspec declarator fun_params ";"
//             | declspec declarator fun_params "{" stmt* "}"
fn parse_fun(it: &mut Tokenizer) -> Expected<Fun> {
  let ty = parse_declspec(it)?;
  let (ret_ty, name) = parse_declarator(it, ty)?;
  let (param_tys, param_names) = parse_fun_params(it)?;
  if consume(it, ";")? {
    Ok(Fun::FunDecl(ret_ty, name, param_tys))
  } else if consume(it, "{")? {
    let mut body = Vec::new();
    while !consume(it, "}")? {
      body.push(parse_stmt(it)?);
    }
    Ok(Fun::FunDef(ret_ty, name, param_tys, param_names, body))
  } else {
    err!("unexpected token, expecting `{` or `;`")
  }
}

// declspec    = "int"
fn parse_declspec(it: &mut Tokenizer) -> Expected<Type> {
  if consume_keyword(it, "int")? {
    Ok(Type::Int)
  } else {
    err!("unexpected token, expecting `int`")
  }
}

fn consume_declspec(it: &mut Tokenizer) -> Expected<Option<Type>> {
  if consume_keyword(it, "int")? {
    Ok(Some(Type::Int))
  } else {
    Ok(None)
  }
}

// declarator  = "*"* ident ("[" num "]")?
fn parse_declarator(it: &mut Tokenizer, mut ty: Type) -> Expected<(Type, String)> {
  while consume(it, "*")? {
    ty = Type::Pointer(Box::new(ty));
  }
  let name = expect_ident(it)?;
  if consume(it, "[")? {
    let n = expect_num(it)?;
    expect(it, "]")?;
    ty = Type::Array(Box::new(ty), n as u32);
  }
  Ok((ty, name))
}

// fun_params  = "(" param (("," param)*)? ")"
fn parse_fun_params(it: &mut Tokenizer) -> Expected<(Vec<Type>, Vec<String>)> {
  expect(it, "(")?;
  let mut params = Vec::new();
  if !consume(it, ")")? {
    params.push(parse_param(it)?);
    while !consume(it, ")")? {
      expect(it, ",")?;
      params.push(parse_param(it)?);
    }
  }
  Ok(params.into_iter().unzip())
}

// param       = declspec declarator
fn parse_param(it: &mut Tokenizer) -> Expected<(Type, String)> {
  let ty = parse_declspec(it)?;
  parse_declarator(it, ty)
}

// stmt        = declspec declarator ("=" expr)? ";"
//             | "if" "(" expr ")" stmt ("else" stmt)?
//             | "for" "(" expr? ";" expr? ";" expr? ")" stmt
//             | "while" "(" expr ")" stmt
//             | "break" ";"
//             | "continue" ";"
//             | "return" expr ";"
//             | "{" stmt* "}"
//             | ";"
//             | expr ";"
fn parse_stmt(it: &mut Tokenizer) -> Expected<Stmt> {
  if let Some(ty) = consume_declspec(it)? {
    let (ty, name) = parse_declarator(it, ty)?;
    let init = if consume(it, "=")? {
      Some(parse_expr(it)?)
    } else {
      None
    };
    expect(it, ";")?;
    Ok(Stmt::VarDef(ty, name, init))
  } else if consume_keyword(it, "if")? {
    expect(it, "(")?;
    let cond = parse_expr(it)?;
    expect(it, ")")?;
    let then_stmt = Box::new(parse_stmt(it)?);
    let else_stmt = if consume_keyword(it, "else")? {
      Some(Box::new(parse_stmt(it)?))
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
    let stmt = Box::new(parse_stmt(it)?);
    Ok(Stmt::For(n1, n2, n3, stmt))
  } else if consume_keyword(it, "while")? {
    expect(it, "(")?;
    let n2 = parse_expr(it).ok();
    expect(it, ")")?;
    let stmt = Box::new(parse_stmt(it)?);
    Ok(Stmt::For(None, n2, None, stmt))
  } else if consume_keyword(it, "break")? {
    expect(it, ";")?;
    Ok(Stmt::Break)
  } else if consume_keyword(it, "continue")? {
    expect(it, ";")?;
    Ok(Stmt::Cont)
  } else if consume_keyword(it, "return")? {
    let n = parse_expr(it)?;
    expect(it, ";")?;
    Ok(Stmt::Return(n))
  } else if consume(it, "{")? {
    let mut stmts = Vec::new();
    while !consume(it, "}")? {
      stmts.push(parse_stmt(it)?);
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

// expr        = ternary
fn parse_expr(it: &mut Tokenizer) -> Expected<AST> {
  parse_ternary(it)
}

// ternary     = assign ("?" expr ":" ternary)?
fn parse_ternary(it: &mut Tokenizer) -> Expected<AST> {
  let cond = parse_assign(it)?;
  if consume(it, "?")? {
    let then = parse_expr(it)?;
    expect(it, ":")?;
    let else_ = parse_ternary(it)?;
    Ok(AST::Ternary(
      Box::new(cond),
      Box::new(then),
      Box::new(else_),
    ))
  } else {
    Ok(cond)
  }
}

// assign      = equality ("=" assign | "+=" assign | "-=" assign | "*=" assign | "/=" assign)?
fn parse_assign(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_equality(it)?;
  if consume(it, "=")? {
    let m = parse_assign(it)?;
    Ok(AST::Assign(Box::new(n), Box::new(m)))
  } else if consume(it, "+=")? {
    // convert x+=y to x=x+y
    let m = parse_assign(it)?;
    let add = AST::Add(Box::new(n.clone()), Box::new(m));
    Ok(AST::Assign(Box::new(n), Box::new(add)))
  } else if consume(it, "-=")? {
    // convert x-=y to x=x-y
    let m = parse_assign(it)?;
    let sub = AST::Sub(Box::new(n.clone()), Box::new(m));
    Ok(AST::Assign(Box::new(n), Box::new(sub)))
  } else if consume(it, "*=")? {
    // convert x*=y to x=x*y
    let m = parse_assign(it)?;
    let mul = AST::Mul(Box::new(n.clone()), Box::new(m));
    Ok(AST::Assign(Box::new(n), Box::new(mul)))
  } else if consume(it, "/=")? {
    // convert x/=y to x=x/y
    let m = parse_assign(it)?;
    let div = AST::Div(Box::new(n.clone()), Box::new(m));
    Ok(AST::Assign(Box::new(n), Box::new(div)))
  } else {
    Ok(n)
  }
}

// equality    = relational ("==" relational | "!=" relational)*
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

// relational  = add ("<" add | "<=" add | ">" add | ">=" add)*
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

// add         = mul ("+" mul | "-" mul)*
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

// mul         = unary ("*" unary | "/" unary)*
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

// unary       = ("+" | "-" | "&" | "*" | "++" | "--") unary
//             | postfix
fn parse_unary(it: &mut Tokenizer) -> Expected<AST> {
  if consume(it, "+")? {
    parse_unary(it)
  } else if consume(it, "-")? {
    let n = AST::Num(0);
    let m = parse_unary(it)?;
    Ok(AST::Sub(Box::new(n), Box::new(m)))
  } else if consume(it, "&")? {
    let n = parse_unary(it)?;
    Ok(AST::Addr(Box::new(n)))
  } else if consume(it, "*")? {
    let n = parse_unary(it)?;
    Ok(AST::Deref(Box::new(n)))
  } else if consume(it, "++")? {
    // convert ++i to i=i+1
    let n = parse_unary(it)?;
    let one = AST::Num(1);
    let add = AST::Add(Box::new(n.clone()), Box::new(one));
    Ok(AST::Assign(Box::new(n), Box::new(add)))
  } else if consume(it, "--")? {
    // convert --i to i=i-1
    let n = parse_unary(it)?;
    let one = AST::Num(1);
    let sub = AST::Sub(Box::new(n.clone()), Box::new(one));
    Ok(AST::Assign(Box::new(n), Box::new(sub)))
  } else {
    parse_postfix(it)
  }
}

// postfix     = primary ("[" expr "]" | "++" | "--")?
fn parse_postfix(it: &mut Tokenizer) -> Expected<AST> {
  let n = parse_primary(it)?;
  if consume(it, "[")? {
    // convert a[i] to *(a+i)
    let m = parse_expr(it)?;
    expect(it, "]")?;
    let add = AST::Add(Box::new(n), Box::new(m));
    Ok(AST::Deref(Box::new(add)))
  } else if consume(it, "++")? {
    // convert i++ to (i=i+1)-1
    let one = AST::Num(1);
    let add = AST::Add(Box::new(n.clone()), Box::new(one.clone()));
    let assign = AST::Assign(Box::new(n), Box::new(add));
    Ok(AST::Sub(Box::new(assign), Box::new(one)))
  } else if consume(it, "--")? {
    // convert i-- to (i=i-1)+1
    let one = AST::Num(1);
    let sub = AST::Sub(Box::new(n.clone()), Box::new(one.clone()));
    let assign = AST::Assign(Box::new(n), Box::new(sub));
    Ok(AST::Add(Box::new(assign), Box::new(one)))
  } else {
    Ok(n)
  }
}

// primary     = "(" expr ")"
//             | ident "(" fun_args
//             | ident
//             | num
fn parse_primary(it: &mut Tokenizer) -> Expected<AST> {
  if consume(it, "(")? {
    let n = parse_expr(it)?;
    expect(it, ")")?;
    Ok(n)
  } else if let Some(name) = consume_ident(it)? {
    if consume(it, "(")? {
      let args = parse_fun_args(it)?;
      Ok(AST::Call(name, args))
    } else {
      Ok(AST::Ident(name))
    }
  } else if let Some(n) = consume_num(it)? {
    Ok(AST::Num(n))
  } else {
    err!("unexpected token, expecting `(`, identifier or number")
  }
}

// fun_args    = (expr ("," expr)*)? ")"
fn parse_fun_args(it: &mut Tokenizer) -> Expected<Vec<AST>> {
  let mut args = Vec::new();
  if consume(it, ")")? {
    Ok(args)
  } else {
    args.push(parse_expr(it)?);
    while !consume(it, ")")? {
      expect(it, ",")?;
      args.push(parse_expr(it)?);
    }
    Ok(args)
  }
}
