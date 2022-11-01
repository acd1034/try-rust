#[derive(Debug)]
struct Person {
  name: String,
  age: u8,
}

struct Unit;

struct Pair(i32, f32);

#[derive(Debug)]
struct Point {
  x: f32,
  y: f32,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Rectangle {
  top_left: Point,
  bottom_right: Point,
}

fn rect_area(
  Rectangle {
    top_left: Point { x: x1, y: y1 },
    bottom_right: Point { x: x2, y: y2 },
  }: Rectangle,
) -> f32 {
  (x2 - x1) * (y1 - y2)
}

fn square(Point { x: x1, y: y1 }: Point, f: f32) -> Rectangle {
  Rectangle {
    top_left: Point { x: x1, y: y1 + f },
    bottom_right: Point { x: x1 + f, y: y1 },
  }
}

fn main() {
  let name = String::from("Peter");
  let age = 27;
  let peter = Person { name, age };

  println!("{:?}", peter);

  let point: Point = Point { x: 10.3, y: 0.4 };

  println!("point coordinates: ({}, {})", point.x, point.y);

  let bottom_right = Point { x: 5.2, y: 0.2 };

  println!("second point: ({}, {})", bottom_right.x, bottom_right.y);

  let Point {
    x: left_edge,
    y: top_edge,
  } = point;

  let rectangle = Rectangle {
    top_left: Point {
      x: left_edge,
      y: top_edge,
    },
    bottom_right: bottom_right,
  };

  println!("{}", rect_area(rectangle));
  println!("{:?}", square(Point { x: 10.3, y: 0.4 }, 0.1));

  let _unit = Unit;

  let pair = Pair(1, 0.1);

  println!("pair contains {:?} and {:?}", pair.0, pair.1);

  let Pair(integer, decimal) = pair;

  println!("pair contains {:?} and {:?}", integer, decimal);
}
