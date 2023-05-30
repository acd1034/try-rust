use crate::common::JoinView;
use crate::ir::function::*;
use crate::ir::inst::*;
use crate::ir::module::*;
use id_arena::Arena;
use std::fmt;

impl fmt::Display for Module {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    irdump(f, self)
  }
}

fn irdump(f: &mut fmt::Formatter, module: &Module) -> fmt::Result {
  write!(f, "; ModuleName = '{}'", module.name())?;
  for (_id, fun) in module.functions().iter() {
    dump_function(f, fun, module.functions())?;
  }
  Ok(())
}

fn dump_function(
  f: &mut fmt::Formatter,
  fun: &Function,
  functions: &Arena<Function>,
) -> fmt::Result {
  let iter = fun.param_tys().iter();
  let param_tys = JoinView::new(iter, ", ");

  if fun.is_declaration() {
    return write!(f, "\n\ndeclare {}({})", fun.name(), param_tys);
  }

  // Emit function return type, name and parameters
  write!(f, "\n\n{}({}):", fun.name(), param_tys)?;

  // Allocate memory
  if fun.memory_arena().len() != 0 {
    let alloca = JoinView::new(0..fun.memory_arena().len(), ",");
    write!(f, "\n  ; alloca={}", alloca)?;
  }

  // Emit function body
  for &block_id in fun.blocks() {
    let block_label = format!(".block{}:", block_id.index());
    // let pred = JoinView::new(fun.get(block_id).pred().iter(), ",");
    // let succ = JoinView::new(fun.get(block_id).succ().iter(), ",");
    write!(f, "\n{}", block_label)?;
    // write!(f, "\n{:<40}; pred={} succ={}", block_label, pred, succ)?;
    for &inst_id in fun.get(block_id).insts() {
      dump_inst(f, &fun.get(inst_id), inst_id, functions)?;
    }
  }

  Ok(())
}

fn dump_inst(
  f: &mut fmt::Formatter,
  inst: &Inst,
  v0: InstId,
  functions: &Arena<Function>,
) -> fmt::Result {
  match inst.kind() {
    InstKind::Eq(v1, v2) => write!(
      f,
      "\n  r{} = eq r{}, r{}",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Ne(v1, v2) => write!(
      f,
      "\n  r{} = ne r{}, r{}",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Lt(v1, v2) => write!(
      f,
      "\n  r{} = lt r{}, r{}",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Le(v1, v2) => write!(
      f,
      "\n  r{} = le r{}, r{}",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Add(v1, v2) => write!(
      f,
      "\n  r{} = add r{}, r{}",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Sub(v1, v2) => write!(
      f,
      "\n  r{} = sub r{}, r{}",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Mul(v1, v2) => write!(
      f,
      "\n  r{} = mul r{}, r{}",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    InstKind::Div(v1, v2) => write!(
      f,
      "\n  r{} = div r{}, r{}",
      v0.index(),
      v1.index(),
      v2.index()
    ),
    // InstKind::Br(v1, block1, block2) => {
    //   write!(f, "\n  br {}, block{}, block{}", v1, block1, block2)
    // }
    // InstKind::Jmp(block1) => write!(f, "\n  jmp block{}", block1),
    // InstKind::Store(m1, v2) => write!(f, "\n  store m{}, {}", m1, v2),
    // InstKind::Load(m1) => write!(f, "\n  r{} = load m{}", v0.index(), m1),
    // InstKind::Call(fun, args) => {
    //   let args = JoinView::new(args.iter(), ", ");
    //   write!(f, "\n  r{} = call {}({});", v0.index(), functions[*fun].name, args)
    // }
    InstKind::Const(n) => write!(f, "\n  r{} = const {}", v0.index(), n),
    // InstKind::Ret(v1) => write!(f, "\n  ret {}", v1),
    _ => todo!(),
  }
}
