use crate::irgen::*;

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
    Inst::Eq(v0, v1, v2) => format!("\n  int {} = {} == {};", v0, v1, v2),
    Inst::Ne(v0, v1, v2) => format!("\n  int {} = {} != {};", v0, v1, v2),
    Inst::Lt(v0, v1, v2) => format!("\n  int {} = {} < {};", v0, v1, v2),
    Inst::Le(v0, v1, v2) => format!("\n  int {} = {} <= {};", v0, v1, v2),
    Inst::Add(v0, v1, v2) => format!("\n  int {} = {} + {};", v0, v1, v2),
    Inst::Sub(v0, v1, v2) => format!("\n  int {} = {} - {};", v0, v1, v2),
    Inst::Mul(v0, v1, v2) => format!("\n  int {} = {} * {};", v0, v1, v2),
    Inst::Div(v0, v1, v2) => format!("\n  int {} = {} / {};", v0, v1, v2),
    Inst::Alloca(..) => String::new(),
    Inst::Store(m1, v2) => format!("\n  m[{}] = {};", m1, v2),
    Inst::Load(v1, m2) => format!("\n  int {} = m[{}];", v1, m2),
    Inst::Ret(v1) => format!("\n  return {};", v1),
  }
}
