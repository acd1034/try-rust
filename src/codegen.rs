use crate::parse::AST;
// use crate::tokenize::Expected;

pub fn codegen(ast: AST) -> String {
  match ast {
    AST::Add(n, m) => format!("({} + {})", codegen(*n), codegen(*m)),
    AST::Sub(n, m) => format!("({} - {})", codegen(*n), codegen(*m)),
    AST::Mul(n, m) => format!("({} * {})", codegen(*n), codegen(*m)),
    AST::Div(n, m) => format!("({} / {})", codegen(*n), codegen(*m)),
    AST::Num(n) => format!("{}", n),
  }
}
