use std::collections::HashMap;

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
