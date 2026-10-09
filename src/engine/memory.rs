use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug)]
pub struct MemoryManager {
    // Placeholder for memory management functionality
    _marker: std::marker::PhantomData<()>,
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

pub type SharedData<T> = Arc<RwLock<T>>;
