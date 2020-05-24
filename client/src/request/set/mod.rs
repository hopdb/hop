mod set_boolean;
mod set_bytes;
mod set_float;
mod set_integer;
mod set_list;
mod set_map;
mod set_set;
mod set_string;
mod set_value;

pub use self::{
    set_boolean::SetBoolean, set_bytes::SetBytes, set_float::SetFloat, set_integer::SetInteger,
    set_list::SetList, set_map::SetMap, set_set::SetSet, set_string::SetString,
    set_value::SetValue,
};

use crate::Backend;
use hop_engine::state::Value;
use std::{iter::FromIterator, sync::Arc};

/// An Set request that hasn't been configured with a value to set.
///
/// This is an intermediary that allows you to cleanly set a value knowing its
/// type from just the method signature, and get back a value in the same type.
///
/// For example, if you call [`SetUnconfigured::bool`], then you will get back a
/// configured [`SetBoolean`] struct which you can `await`. This struct will
/// resolve to a boolean on success. If you call [`SetUnconfigured::int`], then
/// you will get back a [`SetInteger`] which will resolve to an integer when
/// `await`ed.
///
/// # Examples
///
/// Set the key "foo" to a boolean:
///
/// ```
/// use hop::Client;
///
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::memory();
///
/// // we know that it will resolve to a boolean on success
/// let new_value: bool = client.set("foo").bool(true).await?;
///
/// if new_value {
///     println!("Something when the new value is true");
/// } else {
///     println!("Or when it's false");
/// }
/// # Ok(()) }
/// ```
///
/// [`SetUnconfigured::bool`]: #method.bool
/// [`SetUnconfigured::int`]: #method.int
/// [`SetBoolean`]: struct.SetBoolean.html
/// [`SetInteger`]: struct.SetInteger.html
pub struct SetUnconfigured<B: Backend, K: AsRef<[u8]> + Send + Unpin> {
    backend: Arc<B>,
    key: K,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> SetUnconfigured<B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self { backend, key }
    }

    /// An alias for [`bool`].
    ///
    /// [`bool`]: #method.bool
    pub fn boolean(self, boolean: bool) -> SetBoolean<'a, B, K> {
        self.bool(boolean)
    }

    /// Set a key to a boolean.
    ///
    /// The returned struct, when `await`ed, will resolve to a boolean on
    /// success.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to `true`:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").bool(true).await?;
    /// # Ok(()) }
    /// ```
    pub fn bool(self, boolean: bool) -> SetBoolean<'a, B, K> {
        SetBoolean::new(self.backend, self.key, boolean)
    }

    /// Set a key to some bytes.
    ///
    /// The returned struct, when `await`ed, will resolve to a `Vec<u8>` on
    /// success.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to the bytes `[1, 2, 3, 4, 5]`:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").bytes([1u8, 2, 3, 4, 5].as_ref()).await?;
    /// # Ok(()) }
    /// ```
    pub fn bytes(self, bytes: impl Into<Vec<u8>>) -> SetBytes<'a, B, K> {
        SetBytes::new(self.backend, self.key, bytes.into())
    }

    /// Set a key to a float.
    ///
    /// The returned struct, when `await`ed, will resolve to a float on success.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to `1.23`:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").float(1.23).await?;
    /// # Ok(()) }
    /// ```
    pub fn float(self, float: f64) -> SetFloat<'a, B, K> {
        SetFloat::new(self.backend, self.key, float)
    }

    /// An alias for [`int`].
    ///
    /// [`int`]: #method.int
    pub fn integer(self, integer: i64) -> SetInteger<'a, B, K> {
        self.int(integer)
    }

    /// Set a key to an integer.
    ///
    /// The returned struct, when `await`ed, will resolve to an integer on
    /// success.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to `123`:
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").int(123).await?;
    /// # Ok(()) }
    /// ```
    pub fn int(self, integer: i64) -> SetInteger<'a, B, K> {
        SetInteger::new(self.backend, self.key, integer)
    }

    /// Set a key to an list.
    ///
    /// The returned struct, when `await`ed, will resolve to a list on success.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to the list:
    ///
    /// - "foo"
    /// - "bar"
    /// - "baz"
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").list([b"foo".to_vec(), b"bar".to_vec(), b"baz".to_vec()].as_ref()).await?;
    /// # Ok(()) }
    /// ```
    pub fn list(self, list: impl Into<Vec<Vec<u8>>>) -> SetList<'a, B, K> {
        SetList::new(self.backend, self.key, list.into())
    }

    pub fn map<T: IntoIterator<Item = (U, U)>, U: Into<Vec<u8>>>(self, map: T) -> SetMap<'a, B, K> {
        SetMap::new(
            self.backend,
            self.key,
            FromIterator::from_iter(map.into_iter().map(|(k, v)| (k.into(), v.into()))),
        )
    }

    /// Set a key to an list.
    ///
    /// The returned struct, when `await`ed, will resolve to a list on success.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to the set:
    ///
    /// - "foo"
    /// - "bar"
    /// - "foo"
    ///
    /// Then, confirm that it has only 2 items, since there are duplicate
    /// "foo"s.
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// let set = client.set("foo").set([b"foo".to_vec(), b"bar".to_vec(), b"foo".to_vec()].to_vec()).await?;
    ///
    /// assert_eq!(2, set.len());
    /// # Ok(()) }
    /// ```
    pub fn set(self, set: impl Into<Vec<Vec<u8>>>) -> SetSet<'a, B, K> {
        SetSet::new(self.backend, self.key, FromIterator::from_iter(set.into()))
    }

    /// An alias for [`str`].
    ///
    /// [`str`]: #method.str
    #[inline]
    pub fn string(self, string: impl Into<String>) -> SetString<'a, B, K> {
        self.str(string)
    }

    /// Set a key to a string.
    ///
    /// The returned struct, when `await`ed, will resolve to a string on
    /// success.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to the string "bar":
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// client.set("foo").str("bar").await?;
    /// # Ok(()) }
    /// ```
    pub fn str(self, string: impl Into<String>) -> SetString<'a, B, K> {
        SetString::new(self.backend, self.key, string.into())
    }

    /// Set a value to that of a raw engine state value.
    ///
    /// This is mainly useful when you are heavily working with the engine
    /// directly.
    ///
    /// The returned struct, when `await`ed, will resolve to a value that will
    /// be equivalent to the one provided.
    ///
    /// # Examples
    ///
    /// Set the key "foo" to a value containing the string "bar":
    ///
    /// ```
    /// use hop::Client;
    /// use hop_engine::state::Value;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    /// let value = Value::String("bar".to_owned());
    ///
    /// client.set("foo").value(value).await?;
    /// # Ok(()) }
    /// ```
    pub fn value(self, value: impl Into<Value>) -> SetValue<'a, B, K> {
        SetValue::new(self.backend, self.key, value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::SetUnconfigured;
    use crate::backend::MemoryBackend;
    use static_assertions::assert_impl_all;

    assert_impl_all!(SetUnconfigured<MemoryBackend, Vec<u8>>: Send);
}
