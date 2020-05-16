pub mod object;
pub mod value;

pub use self::value::Value;

use self::object::Object;
use alloc::{borrow::ToOwned, string::String, sync::Arc, vec::Vec};
use core::convert::TryFrom;
use dashmap::{mapref::one::RefMut, DashMap};

pub type Key = Vec<u8>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum KeyType {
    Bytes = 0,
    Boolean = 1,
    Float = 2,
    Integer = 3,
    String = 4,
    List = 5,
    Map = 6,
    Set = 7,
}

impl TryFrom<u8> for KeyType {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        use KeyType::*;

        Ok(match v {
            0 => Bytes,
            1 => Boolean,
            2 => Float,
            3 => Integer,
            4 => String,
            5 => List,
            6 => Map,
            7 => Set,
            _ => return Err(()),
        })
    }
}

// The inner map is public to the crate solely for testing purposes.
#[derive(Clone, Debug, Default)]
pub struct State(pub(crate) Arc<DashMap<Key, Value>>);

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve a key's value, providing the default value to insert if the key
    /// doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hop_engine::state::{State, Value};
    ///
    /// let state = State::new();
    /// let key = state.key(b"some:key", Value::boolean);
    ///
    /// match key.value() {
    ///     Value::Boolean(_) => println!("it's a boolean"),
    ///     Value::Set(_) => println!("it's a set"),
    ///     _ => println!("it's something else"),
    /// }
    /// ```
    pub fn key<'a>(&'a self, key: &[u8], f: impl Fn() -> Value) -> RefMut<'a, Key, Value> {
        if key.starts_with(b"__hop__:") {
            panic!("Accessed internal key: {}", String::from_utf8_lossy(key));
        }

        debug_assert!(!key.is_empty());

        loop {
            match self.0.get_mut(key) {
                Some(v) => {
                    break v;
                }
                None => {
                    self.0.insert(key.to_owned(), f());

                    continue;
                }
            }
        }
    }

    /// Retrieve a key's value if it matches a given type.
    ///
    /// If the key exists, but is not the right type, then `None` is returned.
    /// If the key doesn't exist, then the default for the type is inserted
    /// and returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn try_main() -> Option<()> {
    /// use hop_engine::state::{object::{Integer, Object}, State};
    ///
    /// let state = State::new();
    /// // Get the key "some:key" as an integer if it's not already a different
    /// // type.
    /// let mut int = state.typed_key::<Integer>(b"some:key")?;
    ///
    /// *int += 100;
    /// # Some(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn typed_key<'a, K: Object<'a>>(&'a self, key: &[u8]) -> Option<K> {
        let key = self.key(key, K::default);

        if key.value().kind() == K::key_type() {
            Some(K::new(key))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyType, State};
    use core::{convert::TryFrom, fmt::Debug, hash::Hash};
    use static_assertions::assert_impl_all;

    assert_impl_all!(
        KeyType: Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        PartialEq,
        TryFrom<u8>
    );
    assert_impl_all!(State: Clone, Debug, Default);
}
