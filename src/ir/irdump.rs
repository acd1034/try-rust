use crate::common::JoinView;
use crate::ir::function::*;
use crate::ir::inst::*;
use crate::ir::module::*;
use std::fmt;

impl fmt::Display for Module {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    DumpIr::new(self, f).run_on_module()
  }
}

pub struct DumpIr<'a, 'b> {
  module: &'a Module,
  f: &'a mut fmt::Formatter<'b>,
}

impl<'a, 'b> DumpIr<'a, 'b> {
  pub fn new(module: &'a Module, f: &'a mut fmt::Formatter<'b>) -> DumpIr<'a, 'b> {
    DumpIr { module, f }
  }

  pub fn run_on_module(&mut self) -> fmt::Result {
    write!(self.f, "; ModuleName = '{}'", self.module.name())?;
    for (_id, fun) in self.module.functions().iter() {
      self.run_on_function(fun)?;
    }
    Ok(())
  }

  fn run_on_function(&mut self, fun: &Function) -> fmt::Result {
    let iter = fun.param_tys().iter();
    let param_tys = JoinView::new(iter, ", ");

    if fun.is_declaration() {
      return write!(self.f, "\n\ndeclare {}({})", fun.name(), param_tys);
    }

    // Emit function return type, name and parameters
    write!(self.f, "\n\n{}({}):", fun.name(), param_tys)?;

    // Allocate memory
    if fun.memory_arena().len() != 0 {
      write!(self.f, "\n  m = alloca {}", fun.memory_arena().len())?;
    }

    // Emit function body
    for &block_id in fun.blocks() {
      let block_label = format!(".block{}:", block_id.index());
      let pred = JoinView::new(fun.get(block_id).pred().iter().map(|id| id.index()), ",");
      let succ = JoinView::new(fun.get(block_id).succ().iter().map(|id| id.index()), ",");
      write!(self.f, "\n{:<40}; pred={} succ={}", block_label, pred, succ)?;
      for &inst_id in fun.get(block_id).insts() {
        self.run_on_inst(fun.get(inst_id))?;
      }
    }

    Ok(())
  }

  fn run_on_inst(&mut self, inst: &Inst) -> fmt::Result {
    match inst.kind() {
      InstKind::Eq(v0, v1, v2) => write!(
        self.f,
        "\n  r{} = eq r{}, r{}",
        v0.index(),
        v1.index(),
        v2.index()
      ),
      InstKind::Ne(v0, v1, v2) => write!(
        self.f,
        "\n  r{} = ne r{}, r{}",
        v0.index(),
        v1.index(),
        v2.index()
      ),
      InstKind::Lt(v0, v1, v2) => write!(
        self.f,
        "\n  r{} = lt r{}, r{}",
        v0.index(),
        v1.index(),
        v2.index()
      ),
      InstKind::Le(v0, v1, v2) => write!(
        self.f,
        "\n  r{} = le r{}, r{}",
        v0.index(),
        v1.index(),
        v2.index()
      ),
      InstKind::Add(v0, v1, v2) => write!(
        self.f,
        "\n  r{} = add r{}, r{}",
        v0.index(),
        v1.index(),
        v2.index()
      ),
      InstKind::Sub(v0, v1, v2) => write!(
        self.f,
        "\n  r{} = sub r{}, r{}",
        v0.index(),
        v1.index(),
        v2.index()
      ),
      InstKind::Mul(v0, v1, v2) => write!(
        self.f,
        "\n  r{} = mul r{}, r{}",
        v0.index(),
        v1.index(),
        v2.index()
      ),
      InstKind::Div(v0, v1, v2) => write!(
        self.f,
        "\n  r{} = div r{}, r{}",
        v0.index(),
        v1.index(),
        v2.index()
      ),
      InstKind::Load(v0, m1) => write!(self.f, "\n  r{} = load m{}", v0.index(), m1.index()),
      InstKind::Call(v0, fun_id, args) => {
        let args = JoinView::new(args.iter().map(|id| format!("r{}", id.index())), ", ");
        write!(
          self.f,
          "\n  r{} = call {}({});",
          v0.index(),
          self.module.functions_get(*fun_id).name(),
          args
        )
      }
      InstKind::Const(v0, n) => write!(self.f, "\n  r{} = const {}", v0.index(), n),
      InstKind::Br(v1, block1, block2) => {
        write!(
          self.f,
          "\n  br r{}, block{}, block{}",
          v1.index(),
          block1.index(),
          block2.index()
        )
      }
      InstKind::Jmp(block1) => write!(self.f, "\n  jmp block{}", block1.index()),
      InstKind::Store(m1, v2) => write!(self.f, "\n  store m{}, r{}", m1.index(), v2.index()),
      InstKind::Ret(v1) => write!(self.f, "\n  ret r{}", v1.index()),
    }
  }
}
