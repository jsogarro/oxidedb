use crate::types::vector::Vector;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub columns: HashMap<String, Vector>,
    pub column_order: Vec<String>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            column_order: Vec::new(),
        }
    }

    pub fn add_column(&mut self, name: String, data: Vector) {
        if !self.columns.contains_key(&name) {
            self.column_order.push(name.clone());
        }
        self.columns.insert(name, data);
    }

    pub fn row_count(&self) -> usize {
        self.columns.values().next().map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}
