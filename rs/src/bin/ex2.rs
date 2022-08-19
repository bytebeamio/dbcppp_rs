use std::collections::HashMap;

struct S {
    m: HashMap<u64, String>,
}

impl S {
    fn new() -> Self {
        S {
            m: HashMap::new()
        }
    }

    fn f(&self, id: u64) -> usize {
        let s = self.m.get(&id);
    }
}

fn main() {
    m.insert(1, 10);
    let t = m.get(&1);
}