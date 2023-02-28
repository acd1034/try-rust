use crate::common::JoinView;
use crate::ir::*;
use std::fmt;

pub fn codegen(f: &mut fmt::Formatter, module: &Mod) -> fmt::Result {
  write!(f, "// ModuleName = '{}'", module.name)?;
  for fun in &module.funs {
    gen_fun(f, fun, module.funs.as_slice())?;
  }
  Ok(())
}

fn gen_fun(f: &mut fmt::Formatter, fun: &Fun, funs: &[Fun]) -> fmt::Result {
  // Emit function return type, name and parameters
  let params = fun
    .param_tys
    .iter()
    .enumerate()
    .map(|(i, ty)| format!("{} a{}", ty, i));
  let params = JoinView::new(params, ", ");
  write!(f, "\n\n{} {}({}) {{", fun.ret_ty, fun.name, params)?;

  // Allocate memory
  if !fun.mem_arena.is_empty() {
    write!(f, "\n  int m[{}];", fun.mem_arena.len())?;
  }

  // Store function parameters to memory
  for i in 0..fun.param_tys.len() {
    write!(f, "\n  m[{i}] = a{i};")?;
  }

  // Emit function body
  for &bb in &fun.bbs {
    write!(f, "\nbb{}:;", bb)?;
    for &inst in &fun.bb_arena[bb].insts {
      gen_inst(f, &fun.inst_arena[inst], funs)?;
    }
  }

  write!(f, "\n}}")
}

fn gen_inst(f: &mut fmt::Formatter, inst: &Inst, funs: &[Fun]) -> fmt::Result {
  match inst {
    Inst::Eq(r0, v1, v2) => write!(f, "\n  int r{} = {} == {};", r0, v1, v2),
    Inst::Ne(r0, v1, v2) => write!(f, "\n  int r{} = {} != {};", r0, v1, v2),
    Inst::Lt(r0, v1, v2) => write!(f, "\n  int r{} = {} < {};", r0, v1, v2),
    Inst::Le(r0, v1, v2) => write!(f, "\n  int r{} = {} <= {};", r0, v1, v2),
    Inst::Add(r0, v1, v2) => write!(f, "\n  int r{} = {} + {};", r0, v1, v2),
    Inst::Sub(r0, v1, v2) => write!(f, "\n  int r{} = {} - {};", r0, v1, v2),
    Inst::Mul(r0, v1, v2) => write!(f, "\n  int r{} = {} * {};", r0, v1, v2),
    Inst::Div(r0, v1, v2) => write!(f, "\n  int r{} = {} / {};", r0, v1, v2),
    Inst::Br(v1, bb1, bb2) => write!(f, "\n  if ({}) goto bb{}; else goto bb{};", v1, bb1, bb2),
    Inst::Jmp(bb1) => write!(f, "\n  goto bb{};", bb1),
    Inst::Store(m1, v2) => write!(f, "\n  m[{}] = {};", m1, v2),
    Inst::Load(r0, m1) => write!(f, "\n  int r{} = m[{}];", r0, m1),
    Inst::Call(r0, fun, args) => {
      let args = JoinView::new(args.iter(), ", ");
      write!(f, "\n  int r{} = {}({});", r0, funs[*fun].name, args)
    }
    Inst::Ret(v1) => write!(f, "\n  return {};", v1),
  }
}
