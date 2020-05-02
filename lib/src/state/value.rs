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
