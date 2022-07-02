use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct Person {
    id: i64,
    name: String,
}

impl Person {
    pub fn new(id: i64, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}
