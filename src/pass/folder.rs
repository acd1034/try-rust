use crate::ir::{
  builder::*,
  builder_trait::*,
  function::*,
  inst::{Inst, InstId, InstKind},
  module::*,
  visitor_trait::*,
};

pub struct ConstantFolding {
  module: Module,
}

impl ConstantFolding {
  pub fn new(module: Module) -> ConstantFolding {
    ConstantFolding { module }
  }

  pub fn run(mut self) -> Module {
    let fun_ids = self.module.function_ids();
    for fun_id in fun_ids {
      let fun = self.run_on_function(self.module.get_function(fun_id).clone());
      self.module.replace_function(fun_id, fun);
    }
    self.module
  }

  fn run_on_function(&mut self, fun: Function) -> Function {
    let mut builder = Builder::new(fun);
    while let Some(..) = builder.next_block() {
      while let Some(inst_id) = builder.next_inst() {
        if let Some(n) = maybe_fold_const(inst_id, builder.function()) {
          let new_const = builder.build_const(n);
          builder.replace_all_uses(inst_id, new_const);
        }
      }
    }
    builder.retrieve_function()
  }
}

fn maybe_fold_const(inst_id: InstId, fun: &Function) -> Option<i64> {
  use InstKind::*;
  let kind = fun.get(inst_id).kind();
  let (n1, n2) = match kind {
    Eq(v1, v2)
    | Ne(v1, v2)
    | Lt(v1, v2)
    | Le(v1, v2)
    | Add(v1, v2)
    | Sub(v1, v2)
    | Mul(v1, v2)
    | Div(v1, v2) => {
      let n1 = maybe_const(fun.get(*v1))?;
      let n2 = maybe_const(fun.get(*v2))?;
      (n1, n2)
    }
    _ => return None,
  };

  match kind {
    Eq(..) => Some((n1 == n2) as i64),
    Ne(..) => Some((n1 != n2) as i64),
    Lt(..) => Some((n1 < n2) as i64),
    Le(..) => Some((n1 <= n2) as i64),
    Add(..) => Some(n1 + n2),
    Sub(..) => Some(n1 - n2),
    Mul(..) => Some(n1 * n2),
    Div(..) => Some(n1 / n2),
    _ => return None,
  }
}

fn maybe_const(inst: &Inst) -> Option<i64> {
  if let &InstKind::Const(n) = inst.kind() {
    Some(n)
  } else {
    None
  }
}

#[test]
fn test_constant_folding() {
  use crate::irgen::IRGen;
  use crate::parse::parse;
  use crate::pass::{count_ops, DeadCodeElimination};
  use crate::tokenize::Tokenizer;

  let input = r"
int main() {
  int x=4;
  1+2+3+x;
  return x;
}
  ";
  let it = Tokenizer::new(input);
  let funs = parse(it).unwrap();
  let module = IRGen::new("mod".to_string()).irgen(funs).unwrap();

  let fun_id = module.get_function_by_name("main").unwrap();
  let before = count_ops(module.get_function(fun_id));
  let module = ConstantFolding::new(module).run();
  let module = DeadCodeElimination::new(module).run();
  let after = count_ops(module.get_function(fun_id));
  assert!(after < before);
}
