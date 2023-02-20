#[macro_export]
macro_rules! err {
  ($x:expr) => {
    Err(concat!($x, " [", file!(), ":", line!(), "]"))
  };
}

pub type Expected<T> = Result<T, &'static str>;
