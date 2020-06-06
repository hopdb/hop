mod append_bytes;
mod append_list;
mod append_string;

pub use self::{append_bytes::AppendBytes, append_list::AppendList, append_string::AppendString};

use crate::Backend;
use alloc::{string::String, sync::Arc, vec::Vec};

/// A request to append to a key.
pub struct AppendUnconfigured<B: Backend, K: AsRef<[u8]> + Send + Unpin> {
    backend: Arc<B>,
    key: K,
}

impl<'a, B: Backend, K: AsRef<[u8]> + 'a + Send + Unpin> AppendUnconfigured<B, K> {
    pub(crate) fn new(backend: Arc<B>, key: K) -> Self {
        Self { backend, key }
    }

    /// Append bytes to a bytes key.
    ///
    /// The returned struct, when `await`ed, will resolve to the new value on
    /// success.
    ///
    /// # Examples
    ///
    /// Append `[1, 2, 3]` to the key "foo":
    ///
    /// ```
    /// use hop::Client;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::memory();
    ///
    /// client.set("foo").bytes([1u8, 2, 3, 4, 5].as_ref()).await?;
    ///
    /// assert_eq!(8, client.append("foo").bytes([1u8, 2, 3].as_ref()).await?.len());
    /// # Ok(()) }
    /// ```
    pub fn bytes(self, bytes: impl Into<Vec<u8>>) -> AppendBytes<'a, B, K> {
        AppendBytes::new(self.backend, self.key, bytes.into())
    }

    /// Append one or more items to a list.
    pub fn list(self, list: impl Into<Vec<Vec<u8>>>) -> AppendList<'a, B, K> {
        AppendList::new(self.backend, self.key, list.into())
    }

    /// An alias for [`str`].
    ///
    /// [`str`]: #method.str
    #[inline]
    pub fn string(self, string: impl Into<String>) -> AppendString<'a, B, K> {
        self.str(string)
    }

    /// Append to a string.
    pub fn str(self, string: impl Into<String>) -> AppendString<'a, B, K> {
        AppendString::new(self.backend, self.key, string.into())
    }
}

#[cfg(test)]
mod tests {
    use super::AppendUnconfigured;
    use crate::backend::MemoryBackend;
    use alloc::vec::Vec;
    use static_assertions::assert_impl_all;

    assert_impl_all!(AppendUnconfigured<MemoryBackend, Vec<u8>>: Send);
}
