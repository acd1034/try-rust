use std::collections::HashMap;
pub type Expected<T> = Result<T, &'static str>;

enum Node {
  Comma(Box<Node>, Box<Node>),
  Assign(Box<Node>, Box<Node>),
  Ident(String),
  Num(u64),
}

struct CodeGen {
  num: u64,
  vars: HashMap<String, ()>,
}

impl CodeGen {
  fn new() -> CodeGen {
    CodeGen {
      num: 0,
      vars: HashMap::new(),
    }
  }

  fn codegen(&mut self, ast: Node) -> Expected<String> {
    match ast {
      Node::Comma(n, m) => {
        self.num = self.num + 1;
        let lhs = self.codegen(*n)?;
        let rhs = self.codegen(*m)?;
        Ok(format!("({}, {})", lhs, rhs))
      }
      Node::Assign(n, m) => {
        self.num = self.num + 1;
        let rhs = self.codegen(*m)?;
        match *n {
          Node::Ident(name) => match self.vars.get(&name) {
            Some(_var) => Err("variable already defined"),
            None => {
              self.vars.insert(name.clone(), ());
              Ok(format!("({} = {})", name, rhs))
            }
          },
          _ => Err("unexpected rvalue"),
        }
      }
      Node::Ident(name) => {
        self.num = self.num + 1;
        match self.vars.get(&name) {
          Some(_var) => Ok(name),
          None => Err("variables not defined"),
        }
      }
      Node::Num(n) => {
        self.num = self.num + 1;
        Ok(n.to_string())
      }
    }
  }
}

fn main() {
  let lhs = Node::Assign(
    Box::new(Node::Ident("foo".to_string())),
    Box::new(Node::Num(42)),
    // Box::new(Node::Num(42)),
  );
  let rhs = Node::Assign(
    Box::new(Node::Ident("foo".to_string())),
    Box::new(Node::Num(42)),
  );
  // let rhs = Node::Ident("foo".to_string());
  let ast = Node::Comma(Box::new(lhs), Box::new(rhs));
  match CodeGen::new().codegen(ast) {
    Ok(result) => println!("{}", result),
    Err(msg) => println!("error: {}", msg),
  }
}
