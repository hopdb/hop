use super::{Key, KeyType, Value};
use alloc::{string::String, vec::Vec};
use core::ops::{Deref, DerefMut};
use dashmap::{mapref::one::RefMut, DashMap, DashSet};

pub trait Object<'a> {
    fn new(r: RefMut<'a, Vec<u8>, Value>) -> Self;

    fn default() -> Value;

    fn key_type() -> KeyType;
}

pub struct Boolean<'a>(RefMut<'a, Vec<u8>, Value>);

impl<'a> Object<'a> for Boolean<'a> {
    fn new(r: RefMut<'a, Vec<u8>, Value>) -> Self {
        Self(r)
    }

    fn default() -> Value {
        Value::boolean()
    }

    fn key_type() -> KeyType {
        KeyType::Boolean
    }
}

impl<'a> Deref for Boolean<'a> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Value::Boolean(boolean) => boolean,
            _ => unreachable!("didn't get a boolean"),
        }
    }
}

impl<'a> DerefMut for Boolean<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Value::Boolean(boolean) => boolean,
            _ => unreachable!("didn't get a boolean"),
        }
    }
}

pub struct Bytes<'a>(RefMut<'a, Key, Value>);

impl<'a> Object<'a> for Bytes<'a> {
    fn new(r: RefMut<'a, Vec<u8>, Value>) -> Self {
        Self(r)
    }

    fn default() -> Value {
        Value::bytes()
    }

    fn key_type() -> KeyType {
        KeyType::Bytes
    }
}

impl<'a> Deref for Bytes<'a> {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Value::Bytes(bytes) => bytes,
            _ => unreachable!("didn't get a bytes"),
        }
    }
}

impl<'a> DerefMut for Bytes<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Value::Bytes(bytes) => bytes,
            _ => unreachable!("didn't get a bytes"),
        }
    }
}

pub struct Float<'a>(RefMut<'a, Key, Value>);

impl<'a> Object<'a> for Float<'a> {
    fn new(r: RefMut<'a, Vec<u8>, Value>) -> Self {
        Self(r)
    }

    fn default() -> Value {
        Value::float()
    }

    fn key_type() -> KeyType {
        KeyType::Float
    }
}

impl<'a> Deref for Float<'a> {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Value::Float(float) => float,
            _ => unreachable!("didn't get a float"),
        }
    }
}

impl<'a> DerefMut for Float<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Value::Float(float) => float,
            _ => unreachable!("didn't get a float"),
        }
    }
}

pub struct Integer<'a>(RefMut<'a, Key, Value>);

impl<'a> Object<'a> for Integer<'a> {
    fn new(r: RefMut<'a, Vec<u8>, Value>) -> Self {
        Self(r)
    }

    fn default() -> Value {
        Value::integer()
    }

    fn key_type() -> KeyType {
        KeyType::Integer
    }
}

impl<'a> Deref for Integer<'a> {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Value::Integer(integer) => integer,
            _ => unreachable!("didn't get an integer"),
        }
    }
}

impl<'a> DerefMut for Integer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Value::Integer(integer) => integer,
            _ => unreachable!("didn't get an integer"),
        }
    }
}

pub struct List<'a>(RefMut<'a, Key, Value>);

impl<'a> Object<'a> for List<'a> {
    fn new(r: RefMut<'a, Vec<u8>, Value>) -> Self {
        Self(r)
    }

    fn default() -> Value {
        Value::list()
    }

    fn key_type() -> KeyType {
        KeyType::List
    }
}

impl<'a> Deref for List<'a> {
    type Target = Vec<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Value::List(list) => list,
            _ => unreachable!("didn't get an list"),
        }
    }
}

impl<'a> DerefMut for List<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Value::List(list) => list,
            _ => unreachable!("didn't get an integer"),
        }
    }
}

pub struct Map<'a>(RefMut<'a, Key, Value>);

impl<'a> Object<'a> for Map<'a> {
    fn new(r: RefMut<'a, Vec<u8>, Value>) -> Self {
        Self(r)
    }

    fn default() -> Value {
        Value::map()
    }

    fn key_type() -> KeyType {
        KeyType::Map
    }
}

impl<'a> Deref for Map<'a> {
    type Target = DashMap<Vec<u8>, Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Value::Map(map) => map,
            _ => unreachable!("didn't get a map"),
        }
    }
}

impl<'a> DerefMut for Map<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Value::Map(map) => map,
            _ => unreachable!("didn't get a map"),
        }
    }
}

pub struct Set<'a>(RefMut<'a, Key, Value>);

impl<'a> Object<'a> for Set<'a> {
    fn new(r: RefMut<'a, Vec<u8>, Value>) -> Self {
        Self(r)
    }

    fn default() -> Value {
        Value::set()
    }

    fn key_type() -> KeyType {
        KeyType::Set
    }
}

impl<'a> Deref for Set<'a> {
    type Target = DashSet<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Value::Set(set) => set,
            _ => unreachable!("didn't get a set"),
        }
    }
}

impl<'a> DerefMut for Set<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Value::Set(set) => set,
            _ => unreachable!("didn't get a set"),
        }
    }
}

pub struct Str<'a>(RefMut<'a, Key, Value>);

impl<'a> Object<'a> for Str<'a> {
    fn new(r: RefMut<'a, Vec<u8>, Value>) -> Self {
        Self(r)
    }

    fn default() -> Value {
        Value::string()
    }

    fn key_type() -> KeyType {
        KeyType::String
    }
}

impl<'a> Deref for Str<'a> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Value::String(string) => string,
            _ => unreachable!("didn't get a string"),
        }
    }
}

impl<'a> DerefMut for Str<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Value::String(string) => string,
            _ => unreachable!("didn't get a string"),
        }
    }
}
