// seq_apply mf mx = bind mf (f -> bind mx (x -> return (f x)))
fn seq_apply<F, E, T, R>(mf: Result<F, E>, mx: Result<T, E>) -> Result<R, E>
where
  F: FnOnce(T) -> R,
{
  mf.and_then(|f| mx.and_then(|x| Ok(f(x))))
}

// lift_a2 f x y = seq_apply (fmap f x) y
fn lift_a2<F, T, E, U, R>(f: F, mx: Result<T, E>, my: Result<U, E>) -> Result<R, E>
where
  F: FnOnce(T, U) -> R,
{
  seq_apply(mx.map(|x| |y| f(x, y)), my)
}
