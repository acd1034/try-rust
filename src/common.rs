use std::cmp;
use std::collections::HashMap;
use std::fmt;
use std::ops;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! err {
  ($x:expr) => {
    Err(concat!($x, " [", file!(), ":", line!(), "]"))
  };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! err {
  ($x:expr) => {
    Err($x)
  };
}

pub type Expected<T> = Result<T, &'static str>;

// ----- StringRef -----

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StringRef<'a> {
  base: &'a str,
  start: usize,
  end: usize,
}

#[allow(dead_code)]
impl<'a> StringRef<'a> {
  pub fn new(base: &'a str) -> StringRef<'a> {
    StringRef {
      base,
      start: 0,
      end: base.len(),
    }
  }

  pub fn as_str(&self) -> &'a str {
    &self.base[self.start..self.end]
  }

  pub fn substr<Idx: SubStr<StringRef<'a>>>(&self, index: Idx) -> StringRef<'a> {
    index.substr(self)
  }
}

pub trait SubStr<Str>
where
  Str: ?Sized,
{
  fn substr(&self, s: &Str) -> Str;
}

impl<'a> SubStr<StringRef<'a>> for ops::Range<usize> {
  fn substr(&self, s: &StringRef<'a>) -> StringRef<'a> {
    StringRef {
      base: s.base,
      start: s.start + self.start,
      end: cmp::min(s.start + self.end, s.end),
    }
  }
}

impl<'a> SubStr<StringRef<'a>> for ops::RangeFrom<usize> {
  fn substr(&self, s: &StringRef<'a>) -> StringRef<'a> {
    StringRef {
      base: s.base,
      start: s.start + self.start,
      end: s.end,
    }
  }
}

impl<'a> SubStr<StringRef<'a>> for ops::RangeTo<usize> {
  fn substr(&self, s: &StringRef<'a>) -> StringRef<'a> {
    StringRef {
      base: s.base,
      start: s.start,
      end: cmp::min(s.start + self.end, s.end),
    }
  }
}

#[test]
fn test_string_ref() {
  let base = "0123456789";
  {
    let sr = StringRef::new(base).substr(3..7);
    assert_eq!(sr.as_str(), "3456");
  }
  {
    let sr = StringRef::new(base).substr(3..);
    assert_eq!(sr.as_str(), "3456789");
  }
  {
    let sr = StringRef::new(base).substr(..7);
    assert_eq!(sr.as_str(), "0123456");
  }
}

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
