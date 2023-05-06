struct MyList<T> {
  items: Vec<T>,
}

impl<T> MyList<T> {
  fn new() -> MyList<T> {
    MyList { items: Vec::new() }
  }

  fn push(&mut self, item: T) {
    self.items.push(item);
  }

  fn len(&self) -> usize {
    self.items.len()
  }
}

impl<T> std::ops::Index<std::ops::Range<usize>> for MyList<T> {
  type Output = [T];

  fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
    &self.items[index]
  }
}

fn main() {
  let mut list = MyList::new();
  list.push(1);
  list.push(2);
  list.push(3);

  let slice = &list[1..3];
  println!("{:?}", slice);
}
