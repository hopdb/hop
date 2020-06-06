pub mod value;

pub use self::value::Value;

use alloc::{borrow::ToOwned, string::String, sync::Arc, vec::Vec};
use core::convert::TryFrom;
use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};

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

    /// Check if a key exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop_engine::state::{State, Value};
    ///
    /// let state = State::new();
    /// // set a default bytes value to "foo"
    /// state.insert(b"foo".to_vec(), Value::bytes());
    ///
    /// assert!(state.contains_key(b"foo"));
    /// assert!(!state.contains_key(b"bar"));
    /// ```
    pub fn contains_key(&self, key: &[u8]) -> bool {
        self.0.contains_key(key)
    }

    /// Insert a value by key, replacing and returning the existing value if the
    /// key was already taken.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop_engine::state::{State, Value};
    ///
    /// let state = State::new();
    /// assert!(state.insert(b"foo".to_vec(), Value::bytes()).is_none());
    /// assert!(state.insert(b"foo".to_vec(), Value::boolean()).is_some());
    /// ```
    pub fn insert(&self, key: Vec<u8>, value: Value) -> Option<Value> {
        self.0.insert(key, value)
    }

    /// Remove a value by key, returning both the owned key and value if
    /// present.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop_engine::state::{State, Value};
    ///
    /// let state = State::new();
    /// // set a default string value to "foo"
    /// state.key_or_insert_with(b"foo", Value::string);
    ///
    /// assert!(state.contains_key(b"foo"));
    /// assert!(state.remove(b"foo").is_some());
    /// assert!(!state.contains_key(b"foo"));
    /// ```
    pub fn remove(&self, key: &[u8]) -> Option<(Vec<u8>, Value)> {
        self.0.remove(key)
    }

    /// Retrieve an immutable reference to a key-value pair by key.
    ///
    /// Returns `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop_engine::state::{State, Value};
    ///
    /// let state = State::new();
    ///
    /// // the key "foo" does not exist right now
    /// assert!(state.key_ref(b"foo").is_none());
    ///
    /// // but if we insert a key and then check again, it does:
    /// state.insert(b"foo".to_vec(), Value::string());
    /// assert!(state.key_ref(b"foo").is_some());
    /// ```
    pub fn key_ref<'a>(&'a self, key: &[u8]) -> Option<Ref<'a, Key, Value>> {
        if key.starts_with(b"__hop__:") {
            panic!("Accessed internal key: {}", String::from_utf8_lossy(key));
        }

        debug_assert!(!key.is_empty());

        self.0.get(key)
    }

    /// Retrieve a mutable reference to a key-value pair by key.
    ///
    /// Returns `None` if the key does not exist.
    /// ```
    pub fn key_mut<'a>(&'a self, key: &[u8]) -> Option<RefMut<'a, Key, Value>> {
        if key.starts_with(b"__hop__:") {
            panic!("Accessed internal key: {}", String::from_utf8_lossy(key));
        }

        debug_assert!(!key.is_empty());

        self.0.get_mut(key)
    }

    /// Retrieve a key's value, providing a function returning the value to
    /// insert if the key doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hop_engine::state::{State, Value};
    ///
    /// let state = State::new();
    /// let key = state.key_or_insert_with(b"some:key", Value::boolean);
    ///
    /// match key.value() {
    ///     Value::Boolean(_) => println!("it's a boolean"),
    ///     Value::Set(_) => println!("it's a set"),
    ///     _ => println!("it's something else"),
    /// }
    /// ```
    pub fn key_or_insert_with<'a>(
        &'a self,
        key: &[u8],
        f: impl Fn() -> Value,
    ) -> RefMut<'a, Key, Value> {
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

    /// Retrieve the key type of a key's value, if it exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hop_engine::state::{KeyType, State, Value};
    ///
    /// let state = State::new();
    /// assert!(state.key_type(b"foo").is_none());
    ///
    /// state.insert(b"foo".to_vec(), Value::Boolean(true));
    /// assert_eq!(Some(KeyType::Boolean), state.key_type(b"foo"));
    /// ```
    pub fn key_type(&self, key: &[u8]) -> Option<KeyType> {
        self.0.get(key).map(|r| r.value().kind())
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyType, State, Value};
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

    #[test]
    fn test_key_type_nonexistent_key() {
        let state = State::new();
        assert!(state.key_type(b"foo").is_none());
    }

    #[test]
    fn test_key_type_with_key() {
        let state = State::new();
        state.insert(b"foo".to_vec(), Value::Bytes([1, 2].to_vec()));
        assert_eq!(Some(KeyType::Bytes), state.key_type(b"foo"));

        state.insert(b"bar".to_vec(), Value::Integer(123));
        assert_eq!(Some(KeyType::Integer), state.key_type(b"bar"));
    }
}
