use std::collections::HashMap;
use std::fmt;

#[macro_export]
macro_rules! err {
  ($x:expr) => {
    Err(concat!($x, " [", file!(), ":", line!(), "]"))
  };
}

pub type Expected<T> = Result<T, &'static str>;

// ----- JoinView -----

pub struct JoinView<'a, I: Clone + Iterator>
where
  I::Item: fmt::Display,
{
  iter: I,
  dlm: &'a str,
}

impl<'a, I: Clone + Iterator> JoinView<'a, I>
where
  I::Item: fmt::Display,
{
  pub fn new(iter: I, dlm: &'a str) -> JoinView<'a, I> {
    JoinView { iter, dlm }
  }
}

impl<'a, I: Clone + Iterator> fmt::Display for JoinView<'a, I>
where
  I::Item: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut iter = self.iter.clone();
    if let Some(item) = iter.next() {
      write!(f, "{}", item)?;
      for item in iter {
        write!(f, "{}{}", self.dlm, item)?;
      }
    }
    Ok(())
  }
}

// ----- Scope -----

pub struct Scope<V> {
  vars: Vec<HashMap<String, V>>,
}

impl<V> Scope<V> {
  pub fn new() -> Scope<V> {
    let vars = Vec::new();
    Scope { vars }
  }

  pub fn push(&mut self) {
    self.vars.push(HashMap::new());
  }

  pub fn pop(&mut self) {
    self.vars.pop();
  }

  pub fn insert(&mut self, k: String, v: V) -> Option<V> {
    self.vars.last_mut().unwrap().insert(k, v)
  }

  pub fn get(&self, k: &str) -> Option<&V> {
    self.vars.last().unwrap().get(k)
  }

  pub fn get_all(&self, k: &str) -> Option<&V> {
    for vars in self.vars.iter().rev() {
      let var = vars.get(k);
      if var.is_some() {
        return var;
      }
    }
    None
  }
}
