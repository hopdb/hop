mod context;

pub use context::{Context, ParseError};

use super::CommandId;
use crate::state::KeyType;
use alloc::vec::Vec;
use core::slice::SliceIndex;

#[derive(Debug)]
pub struct Request {
    args: Option<Vec<Vec<u8>>>,
    key_type: Option<KeyType>,
    kind: CommandId,
}

impl Request {
    pub fn new(kind: CommandId, args: Option<Vec<Vec<u8>>>) -> Self {
        Self {
            args,
            key_type: None,
            kind,
        }
    }

    pub fn new_with_type(kind: CommandId, args: Option<Vec<Vec<u8>>>, key_type: KeyType) -> Self {
        Self {
            args,
            key_type: Some(key_type),
            kind,
        }
    }

    pub fn args(&self) -> Option<&[Vec<u8>]> {
        self.args.as_deref()
    }

    pub fn arg<I: SliceIndex<[Vec<u8>]>>(
        &self,
        index: I,
    ) -> Option<&<I as SliceIndex<[Vec<u8>]>>::Output> {
        let args = self.args.as_ref()?;

        let refs = args.get(index)?;

        Some(refs)
    }

    pub fn flatten_args(&self) -> Option<Vec<u8>> {
        let start = if self.kind.has_key() { 1 } else { 0 };

        Some(
            self.args
                .as_ref()?
                .get(start..)?
                .iter()
                .fold(Vec::new(), |mut acc, arg| {
                    acc.extend_from_slice(arg);

                    acc
                }),
        )
    }

    pub fn key(&self) -> Option<&[u8]> {
        if !self.kind.has_key() {
            return None;
        }

        self.args
            .as_ref()
            .and_then(|args| args.get(0).map(|x| x.as_slice()))
    }

    /// Returns the requested type of key to work with, if any.
    ///
    /// Some commands only work with one type of key, such as a boolean, where
    /// this isn't taken into account. Other commands, such as [`Append`], can
    /// work with bytes, lists, and strings in unique ways. Commands like
    /// `Append` check the key type to know what type of key to work with.
    ///
    /// [`Append`]: impl/struct.Append.html
    pub fn key_type(&self) -> Option<KeyType> {
        self.key_type
    }

    pub fn kind(&self) -> CommandId {
        self.kind
    }

    pub fn into_args(mut self) -> Option<Vec<Vec<u8>>> {
        self.args.take()
    }
}
