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

    pub fn as_boolean_ref(&self) -> Option<&bool> {
        match self {
            Self::Boolean(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_boolean_mut(&mut self) -> Option<&mut bool> {
        match self {
            Self::Boolean(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    pub fn bytes() -> Self {
        Self::Bytes(Vec::new())
    }

    pub fn as_bytes_ref(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            Self::Bytes(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_bytes(&self) -> bool {
        matches!(self, Value::Bytes(_))
    }

    pub fn float() -> Self {
        Self::Float(0.0)
    }

    pub fn as_float_ref(&self) -> Option<&f64> {
        match self {
            Self::Float(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_float_mut(&mut self) -> Option<&mut f64> {
        match self {
            Self::Float(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn integer() -> Self {
        Self::Integer(0)
    }

    pub fn as_integer_ref(&self) -> Option<&i64> {
        match self {
            Self::Integer(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_integer_mut(&mut self) -> Option<&mut i64> {
        match self {
            Self::Integer(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    pub fn list() -> Self {
        Self::List(Vec::new())
    }

    pub fn as_list_ref(&self) -> Option<&[Vec<u8>]> {
        match self {
            Self::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Vec<u8>>> {
        match self {
            Self::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    pub fn map() -> Self {
        Self::Map(DashMap::new())
    }

    pub fn as_map_ref(&self) -> Option<&DashMap<Vec<u8>, Vec<u8>>> {
        match self {
            Self::Map(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut DashMap<Vec<u8>, Vec<u8>>> {
        match self {
            Self::Map(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Value::Map(_))
    }

    pub fn set() -> Self {
        Self::Set(DashSet::new())
    }

    pub fn as_set_ref(&self) -> Option<&DashSet<Vec<u8>>> {
        match self {
            Self::Set(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_set_mut(&mut self) -> Option<&mut DashSet<Vec<u8>>> {
        match self {
            Self::Set(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_set(&self) -> bool {
        matches!(self, Value::Set(_))
    }

    pub fn string() -> Self {
        Self::String(String::new())
    }

    pub fn as_string_ref(&self) -> Option<&str> {
        match self {
            Self::String(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::String(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Boolean(_))
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
