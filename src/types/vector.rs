use crate::types::atom::Atom;

#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    pub data: Vec<Atom>,
    pub type_code: i8,
}

impl Vector {
    pub fn new(data: Vec<Atom>) -> Self {
        let type_code = if data.is_empty() {
            0 // mixed list
        } else {
            data[0].type_code()
        };

        Self { data, type_code }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
