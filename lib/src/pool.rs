use alloc::{
    boxed::Box,
    vec::Vec,
};
use core::fmt::{Debug, Formatter, Result as FmtResult};


// A growable pool of items to be used and then pushed back into the pool.
//
// This is intended for single-threaded use, where the same items can be used
// over and over across instantiations of something.
pub struct Pool<T> {
    init: Box<dyn Fn() -> T + Send + 'static>,
    items: Vec<T>,
}

impl<T> Pool<T> {
    pub fn new(initial_size: usize, init: impl Fn() -> T + Send + 'static) -> Self {
        let mut items = Vec::new();

        for _ in 0..initial_size {
            items.push(init());
        }

        Self {
            init: Box::new(init),
            items,
        }
    }

    /// Pulls an item out of the pool, instantiating another if necessary.
    pub fn pull(&mut self) -> T {
        self.items.pop().unwrap_or_else(&self.init)
    }

    /// Pushes an item back into the pool.
    ///
    /// The item must have originally been from the pool, or have its position
    /// in the pool replaced by the new item. If more items are pushed onto the
    /// pool than the defined capacity,
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }
}

impl<T: Debug> Debug for Pool<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Pool")
            .field("init", &"function to init items")
            .field("items", &self.items)
            .finish()
    }
}
