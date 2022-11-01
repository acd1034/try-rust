use std::mem;

// This function borrows a slice
// この関数はスライスを借用する
fn analyze_slice(slice: &[i32]) {
  println!("first element of the slice: {}", slice[0]);
  println!("the slice has {} elements", slice.len());
  for x in slice {
    println!("{}", x);
  }
}

fn main() {
  // Fixed-size array (type signature is superfluous)
  // 固定長の配列（型シグネチャは冗長なので、なくても可）
  let xs: [i32; 5] = [1, 2, 3, 4, 5];

  // All elements can be initialized to the same value
  // すべての要素を0にする場合
  let ys: [i32; 500] = [0; 500];

  // Indexing starts at 0
  // インデックスは０から
  println!("first element of the array: {}", xs[0]);
  println!("second element of the array: {}", xs[1]);

  // `len` returns the count of elements in the array
  // `len`は配列の要素数を返す。
  println!("number of elements in array: {}", xs.len());

  // Arrays are stack allocated
  // 配列はスタック上に置かれる
  println!("array occupies {} bytes", mem::size_of_val(&xs));

  // Arrays can be automatically borrowed as slices
  // 配列は自動的にスライスとして借用される。
  println!("borrow the whole array as a slice");
  analyze_slice(&xs);

  // Slices can point to a section of an array
  // They are of the form [starting_index..ending_index]
  // starting_index is the first position in the slice
  // ending_index is one more than the last position in the slice
  // スライスは配列の一部を指すことができる。
  // [starting_index..ending_index] の形をとり、
  // starting_index はスライスの先頭の位置を表し、
  // ending_index はスライスの末尾の1つ先の位置を表す。
  println!("borrow a section of the array as a slice");
  analyze_slice(&ys[1..4]);

  // Out of bound indexing causes compile error
  // インデックスが範囲外のときはコンパイルエラー
  // println!("{}", xs[5]);
}
