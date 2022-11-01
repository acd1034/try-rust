use std::fmt::{self, Display, Formatter};

// タプルを関数の引数及び返り値として使用している。
fn reverse(pair: (i32, bool)) -> (bool, i32) {
  // `let`でタプルの中の値を別の変数に束縛することができる。
  let (integer, boolean) = pair;

  (boolean, integer)
}

// 以下の構造体は後ほど「演習」で用いる。
#[derive(Debug)]
struct Matrix(f32, f32, f32, f32);

impl Display for Matrix {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "({} {})\n({} {})", self.0, self.1, self.2, self.3)
  }
}

fn transpose(m: Matrix) -> Matrix {
  Matrix(m.0, m.2, m.1, m.3)
}

fn main() {
  // 様々な型を値に持つタプル
  let long_tuple = (
    1u8, 2u16, 3u32, 4u64, -1i8, -2i16, -3i32, -4i64, 0.1f32, 0.2f64, 'a', true,
  );

  // インデックスを用いて、タプル内の要素を参照できる。
  println!("long tuple first value: {}", long_tuple.0);
  println!("long tuple second value: {}", long_tuple.1);

  // タプルはタプルのメンバになれる
  let tuple_of_tuples = ((1u8, 2u16, 2u32), (4u64, -1i8), -2i16);

  // タプルはプリント可能である。
  println!("tuple of tuples: {:?}", tuple_of_tuples);

  // しかし長すぎる(長さ13以上の)タプルはプリントできない
  // let too_long_tuple = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
  // println!("too long tuple: {:?}", too_long_tuple);
  // TODO ^ 上記2行のコメントを外して、コンパイルエラーになることを確認

  let pair = (1, true);
  println!("pair is {:?}", pair);

  println!("the reversed pair is {:?}", reverse(pair));

  // 要素を1つしか持たないタプルを作成する場合、括弧で囲まれたただのリテラル
  // と区別するため、カンマが必要になる。
  println!("one element tuple: {:?}", (5u32,));
  println!("just an integer: {:?}", (5u32));

  //タプルを分解して別の変数にそれぞれの値を代入
  let tuple = (1, "hello", 4.5, true);

  let (a, b, c, d) = tuple;
  println!("{:?}, {:?}, {:?}, {:?}", a, b, c, d);

  let matrix = Matrix(1.1, 1.2, 2.1, 2.2);
  println!("{:}", matrix);
  println!("{:}", transpose(matrix));
  // println!("{:}", matrix);
}
