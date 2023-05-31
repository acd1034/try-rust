use crate::common::JoinView;
use crate::ir::{function::*, inst::*, module::*};
use id_arena::Arena;
use std::fmt;

pub fn codegen(f: &mut fmt::Formatter, module: &Module) -> fmt::Result {
  write!(f, "// ModuleName = '{}'", module.name())?;
  for (_id, fun) in module.functions() {
    gen_fun(f, fun, module.functions())?;
  }
  Ok(())
}

fn gen_fun(f: &mut fmt::Formatter, fun: &Function, funs: &Arena<Function>) -> fmt::Result {
  if fun.is_declaration() {
    // Emit function declaration
    let iter = fun.param_tys().iter().map(|ty| format!("{}", ty));
    let param_tys = JoinView::new(iter, ", ");
    write!(f, "\n\n{} {}({});", fun.ret_ty(), fun.name(), param_tys)
  } else {
    // Emit function return type, name and parameters
    let iter = fun
      .param_tys()
      .iter()
      .enumerate()
      .map(|(i, ty)| format!("{} a{}", ty, i));
    let param_tys = JoinView::new(iter, ", ");
    write!(f, "\n\n{} {}({}) {{", fun.ret_ty(), fun.name(), param_tys)?;

    // Allocate memory
    if fun.memory_arena().len() != 0 {
      write!(f, "\n  int m[{}];", fun.memory_arena().len())?;
    }

    // Store function parameters to memory
    for i in 0..fun.param_tys().len() {
      write!(f, "\n  m[{i}] = a{i};")?;
    }

    // Emit function body
    for &block_id in fun.blocks() {
      write!(f, "\nblock{}:;", block_id.index())?;
      for &inst_id in fun.get(block_id).insts() {
        gen_inst(f, fun.get(inst_id), funs)?;
      }
    }

    write!(f, "\n}}")
  }
}

fn gen_inst(f: &mut fmt::Formatter, inst: &Inst, funs: &Arena<Function>) -> fmt::Result {
  match inst.kind() {
    InstKind::Eq(v0, v1, v2) => write!(
      f,
      "\n  int r{} = r{} == r{};",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Ne(v0, v1, v2) => write!(
      f,
      "\n  int r{} = r{} != r{};",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Lt(v0, v1, v2) => write!(
      f,
      "\n  int r{} = r{} < r{};",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Le(v0, v1, v2) => write!(
      f,
      "\n  int r{} = r{} <= r{};",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Add(v0, v1, v2) => write!(
      f,
      "\n  int r{} = r{} + r{};",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Sub(v0, v1, v2) => write!(
      f,
      "\n  int r{} = r{} - r{};",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Mul(v0, v1, v2) => write!(
      f,
      "\n  int r{} = r{} * r{};",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Div(v0, v1, v2) => write!(
      f,
      "\n  int r{} = r{} / r{};",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Load(v0, m1) => write!(f, "\n  int r{} = m[{}];", v0.index(), m1.index()),
    InstKind::Call(v0, fun, args) => {
      let args = JoinView::new(args.iter().map(|id| format!("r{}", id.index())), ", ");
      write!(
        f,
        "\n  int r{} = {}({});",
        v0.index(),
        funs.get(*fun).unwrap().name(),
        args
      )
    }
    InstKind::Const(v0, n) => write!(f, "\n  int r{} = {};", v0.index(), n),
    InstKind::Br(v1, block1, block2) => write!(
      f,
      "\n  if (r{}) goto block{}; else goto block{};",
      v1.index(),
      block1.index(),
      block2.index()
    ),
    InstKind::Jmp(block1) => write!(f, "\n  goto block{};", block1.index()),
    InstKind::Store(m1, v2) => write!(f, "\n  m[{}] = r{};", m1.index(), v2.index()),
    InstKind::Ret(v1) => write!(f, "\n  return r{};", v1.index()),
  }
}
