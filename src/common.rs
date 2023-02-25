use std::fmt;

#[macro_export]
macro_rules! err {
  ($x:expr) => {
    Err(concat!($x, " [", file!(), ":", line!(), "]"))
  };
}

pub type Expected<T> = Result<T, &'static str>;

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
