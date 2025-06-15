use std::collections::HashMap;
use crate::types::atom::Atom;

#[derive(Debug, Clone, PartialEq)]
pub struct Dictionary {
    pub data: HashMap<String, Atom>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: Atom) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&Atom> {
        self.data.get(key)
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}