use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Node {
  data: i32,
  parent: Option<Weak<RefCell<Node>>>,
  children: Vec<Rc<RefCell<Node>>>,
}

impl Node {
  fn new(data: i32) -> Rc<RefCell<Node>> {
    Rc::new(RefCell::new(Node {
      data,
      parent: None,
      children: Vec::new(),
    }))
  }

  fn pretty_print(&self) {
    if let Some(w) = &self.parent {
      if let Some(cell) = w.upgrade() {
        println!("%{} = %{}", self.data, cell.borrow().data);
      }
    }
  }
}

fn pretty_print(graph: &[Rc<RefCell<Node>>]) {
  for item in graph {
    item.borrow().pretty_print();
  }
}

fn connect_nodes(parent: &Rc<RefCell<Node>>, child: &Rc<RefCell<Node>>) {
  parent.borrow_mut().children.push(child.clone());
  child.borrow_mut().parent = Some(Rc::downgrade(parent));
}

fn main() {
  let graph = vec![Node::new(1), Node::new(2), Node::new(3)];
  connect_nodes(&graph[0], &graph[1]);
  connect_nodes(&graph[0], &graph[2]);
  pretty_print(&graph);
}
