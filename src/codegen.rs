use crate::parse::AST;
use crate::tokenize::Expected;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::support::LLVMString;
use inkwell::values::{AnyValue, IntValue};

pub struct CodeGen<'ctx> {
  pub context: &'ctx Context,
  pub module: Module<'ctx>,
  pub builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
  fn codegen_impl(&self, ast: AST) -> Expected<IntValue> {
    let i64_type = self.context.i64_type();
    match ast {
      AST::Add(n, m) => Err("unimplemented!"),
      AST::Sub(n, m) => Err("unimplemented!"),
      AST::Mul(n, m) => Err("unimplemented!"),
      AST::Div(n, m) => Err("unimplemented!"),
      AST::Num(n) => Ok(i64_type.const_int(n, false)),
    }
  }

  pub fn codegen(&self, ast: AST) -> Expected<String> {
    let i64_type = self.context.i64_type();
    let fn_type = i64_type.fn_type(&[], false);
    let function = self.module.add_function("main", fn_type, None);
    let basic_block = self.context.append_basic_block(function, "entry");

    self.builder.position_at_end(basic_block);

    let i64_value = self.codegen_impl(ast)?;

    self.builder.build_return(Some(&i64_value));

    if function.verify(true) {
      Ok(function.print_to_string().to_string())
    } else {
      // unsafe { function.delete(); }
      Err("postprocess: failed to verify function")
    }
  }
}
