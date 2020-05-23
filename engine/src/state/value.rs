use super::KeyType;
use alloc::{string::String, vec::Vec};
use dashmap::{DashMap, DashSet};

#[derive(Debug)]
pub enum Value {
    Boolean(bool),
    Bytes(Vec<u8>),
    Float(f64),
    Integer(i64),
    List(Vec<Vec<u8>>),
    Map(DashMap<Vec<u8>, Vec<u8>>),
    Set(DashSet<Vec<u8>>),
    String(String),
}

impl Value {
    pub fn kind(&self) -> KeyType {
        match self {
            Self::Boolean(_) => KeyType::Boolean,
            Self::Bytes(_) => KeyType::Bytes,
            Self::Float(_) => KeyType::Float,
            Self::Integer(_) => KeyType::Integer,
            Self::List(_) => KeyType::List,
            Self::Map(_) => KeyType::Map,
            Self::Set(_) => KeyType::Set,
            Self::String(_) => KeyType::String,
        }
    }

    pub fn boolean() -> Self {
        Self::Boolean(false)
    }

    pub fn bytes() -> Self {
        Self::Bytes(Vec::new())
    }

    pub fn float() -> Self {
        Self::Float(0.0)
    }

    pub fn integer() -> Self {
        Self::Integer(0)
    }

    pub fn list() -> Self {
        Self::List(Vec::new())
    }

    pub fn map() -> Self {
        Self::Map(DashMap::new())
    }

    pub fn set() -> Self {
        Self::Set(DashSet::new())
    }

    pub fn string() -> Self {
        Self::String(String::new())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<Vec<Vec<u8>>> for Value {
    fn from(value: Vec<Vec<u8>>) -> Self {
        Self::List(value)
    }
}

impl From<DashMap<Vec<u8>, Vec<u8>>> for Value {
    fn from(value: DashMap<Vec<u8>, Vec<u8>>) -> Self {
        Self::Map(value)
    }
}

impl From<DashSet<Vec<u8>>> for Value {
    fn from(value: DashSet<Vec<u8>>) -> Self {
        Self::Set(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Value;
    use alloc::{string::String, vec::Vec};
    use core::fmt::Debug;
    use dashmap::{DashMap, DashSet};
    use static_assertions::assert_impl_all;

    assert_impl_all!(
        Value: Debug,
        From<bool>,
        From<Vec<u8>>,
        From<f64>,
        From<i64>,
        From<Vec<Vec<u8>>>,
        From<DashMap<Vec<u8>, Vec<u8>>>,
        From<Vec<u8>>,
        From<DashSet<Vec<u8>>>,
        From<String>,
    );
}
