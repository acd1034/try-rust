use crate::ir::*;

pub fn codegen(module: &Mod) -> String {
  let mut ret = format!("// ModuleName = '{}'", module.name);
  ret += "\nint m[1 << 16];";
  for fun in &module.funs {
    ret += gen_fun(fun).as_str();
  }
  ret
}

fn gen_fun(fun: &Fun) -> String {
  let mut ret = format!("\n\nint {}() {{", fun.name);
  for &bb in &fun.bbs {
    ret += format!("\nbb{}:;", bb).as_str();
    for &inst in &fun.bb_arena[bb].insts {
      ret += gen_inst(&fun.inst_arena[inst]).as_str();
    }
  }
  ret += "\n}";
  ret
}

fn gen_inst(inst: &Inst) -> String {
  match inst {
    Inst::Eq(r0, v1, v2) => format!("\n  int r{} = {} == {};", r0, v1, v2),
    Inst::Ne(r0, v1, v2) => format!("\n  int r{} = {} != {};", r0, v1, v2),
    Inst::Lt(r0, v1, v2) => format!("\n  int r{} = {} < {};", r0, v1, v2),
    Inst::Le(r0, v1, v2) => format!("\n  int r{} = {} <= {};", r0, v1, v2),
    Inst::Add(r0, v1, v2) => format!("\n  int r{} = {} + {};", r0, v1, v2),
    Inst::Sub(r0, v1, v2) => format!("\n  int r{} = {} - {};", r0, v1, v2),
    Inst::Mul(r0, v1, v2) => format!("\n  int r{} = {} * {};", r0, v1, v2),
    Inst::Div(r0, v1, v2) => format!("\n  int r{} = {} / {};", r0, v1, v2),
    Inst::Br(v1, bb1, bb2) => format!("\n  if ({}) goto bb{}; else goto bb{};", v1, bb1, bb2),
    Inst::Jmp(bb1) => format!("\n  goto bb{};", bb1),
    Inst::Store(m1, v2) => format!("\n  m[{}] = {};", m1, v2),
    Inst::Load(r0, m1) => format!("\n  int r{} = m[{}];", r0, m1),
    Inst::Ret(v1) => format!("\n  return {};", v1),
  }
}
