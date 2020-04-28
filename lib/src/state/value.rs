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

// impl Eq for Object { }

// impl PartialEq for Object {
//     fn eq(&self, other: &Self) -> bool {
//         use Object::*;

//         // This can be a macro but macros aren't great.
//         //
//         // Match on the case of where `self` and `other` are the same variant,
//         // and then check that their values are equivalent.
//         //
//         // If one is of one type and the other, well, another, then they're
//         // obviously not equivalent.
//         match (self, other) {
//             (Boolean(a), Boolean(b)) => a == b,
//             (Boolean(_), _) | (_, Boolean(_)) => false,
//             (Bytes(a), Bytes(b)) => a == b,
//             (Bytes(_), _) | (_, Bytes(_)) => false,
//             // FIXME: Don't strictly check like this.
//             (Float(a), Float(b)) => a == b,
//             (Float(_), _) | (_, Float(_)) => false,
//             (Integer(a), Integer(b)) => a == b,
//             (Integer(_), _) | (_, Integer(_)) => false,
//             (List(a), List(b)) => a == b,
//             (List(_), _) | (_, List(_)) => false,
//             (Map(a), Map(b)) => a == b,
//             (Map(_), _) | (_, Map(_)) => false,
//             (Set(a), Set(b)) => a == b,
//             (Set(_), _) | (_, Set(_)) => false,
//             (String(a), String(b)) => a == b,
//         }
//     }
// }
