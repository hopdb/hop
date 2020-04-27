pub mod error;
pub mod object;

use alloc::{
    borrow::ToOwned,
    sync::Arc,
    string::String,
    vec::Vec,
};
use core::{
    convert::TryFrom,
    ops::{Deref, DerefMut},
};
use dashmap::{
    mapref::one::RefMut,
    DashMap,
    DashSet,
};
use self::{
    error::{RetrievalError, Result},
    object::Object,
};

#[derive(Clone, Debug)]
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

pub struct Boolean<'a>(RefMut<'a, Vec<u8>, Object>);

impl<'a> Deref for Boolean<'a> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Object::Boolean(boolean) => boolean,
            _ => unreachable!("didn't get a boolean"),
        }
    }
}

impl<'a> DerefMut for Boolean<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Object::Boolean(boolean) => boolean,
            _ => unreachable!("didn't get a boolean"),
        }
    }
}

pub struct Bytes<'a>(RefMut<'a, Vec<u8>, Object>);

impl<'a> Deref for Bytes<'a> {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Object::Bytes(bytes) => bytes,
            _ => unreachable!("didn't get a bytes"),
        }
    }
}

impl<'a> DerefMut for Bytes<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Object::Bytes(bytes) => bytes,
            _ => unreachable!("didn't get a bytes"),
        }
    }
}

pub struct Float<'a>(RefMut<'a, Vec<u8>, Object>);

impl<'a> Deref for Float<'a> {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Object::Float(float) => float,
            _ => unreachable!("didn't get a float"),
        }
    }
}

impl<'a> DerefMut for Float<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Object::Float(float) => float,
            _ => unreachable!("didn't get a float"),
        }
    }
}

pub struct Integer<'a>(RefMut<'a, Vec<u8>, Object>);

impl<'a> Deref for Integer<'a> {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Object::Integer(integer) => integer,
            _ => unreachable!("didn't get an integer"),
        }
    }
}

impl<'a> DerefMut for Integer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Object::Integer(integer) => integer,
            _ => unreachable!("didn't get an integer"),
        }
    }
}

pub struct List<'a>(RefMut<'a, Vec<u8>, Object>);

impl<'a> Deref for List<'a> {
    type Target = Vec<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Object::List(list) => list,
            _ => unreachable!("didn't get an list"),
        }
    }
}

impl<'a> DerefMut for List<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Object::List(list) => list,
            _ => unreachable!("didn't get an integer"),
        }
    }
}

pub struct Map<'a>(RefMut<'a, Vec<u8>, Object>);

impl<'a> Deref for Map<'a> {
    type Target = DashMap<Vec<u8>, Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Object::Map(map) => map,
            _ => unreachable!("didn't get a map"),
        }
    }
}

impl<'a> DerefMut for Map<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Object::Map(map) => map,
            _ => unreachable!("didn't get a map"),
        }
    }
}

pub struct Set<'a>(RefMut<'a, Vec<u8>, Object>);

impl<'a> Deref for Set<'a> {
    type Target = DashSet<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Object::Set(set) => set,
            _ => unreachable!("didn't get a set"),
        }
    }
}

impl<'a> DerefMut for Set<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Object::Set(set) => set,
            _ => unreachable!("didn't get a set"),
        }
    }
}

pub struct Str<'a>(RefMut<'a, Vec<u8>, Object>);

impl<'a> Deref for Str<'a> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self.0.value() {
            Object::String(string) => string,
            _ => unreachable!("didn't get a string"),
        }
    }
}

impl<'a> DerefMut for Str<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.value_mut() {
            Object::String(string) => string,
            _ => unreachable!("didn't get a string"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct State(Arc<DashMap<Vec<u8>, Object>>);

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

impl State {
    pub fn key<'a>(&'a self, key: &[u8], f: impl Fn() -> Object) -> RefMut<'a, Vec<u8>, Object> {
        if key.starts_with(b"__hop__:") {
            panic!("Accessed internal key: {}", String::from_utf8_lossy(key));
        }

        debug_assert!(!key.is_empty());

        loop {
            match self.0.get_mut(key) {
                Some(v) => return v,
                None => {
                    self.0.insert(key.to_owned(), f());

                    continue;
                },
            }
        }
    }

    pub fn key_optional<'a>(&'a self, key: &[u8]) -> Option<RefMut<'a, Vec<u8>, Object>> {
        if key.starts_with(b"__hop__:") {
            panic!("Accessed internal key: {}", String::from_utf8_lossy(key));
        }

        debug_assert!(!key.is_empty());

        self.0.get_mut(key)
    }

    pub fn bool(&self, key: &[u8]) -> Result<Boolean<'_>> {
        let mut r = self.key(key, Object::boolean);

        match r.value_mut() {
            Object::Boolean(_) => Ok(Boolean(r)),
            _ => Err(RetrievalError::TypeWrong),
        }
    }

    pub fn int(&self, key: &[u8]) -> Result<Integer<'_>> {
        let mut r = self.key(key, Object::integer);

        match r.value_mut() {
            Object::Integer(_) => Ok(Integer(r)),
            _ => Err(RetrievalError::TypeWrong),
        }
    }

    pub fn bytes(&self, key: &[u8]) -> Result<Bytes<'_>> {
        let mut r = self.key(key, Object::bytes);

        match r.value_mut() {
            Object::Bytes(_) => Ok(Bytes(r)),
            _ => Err(RetrievalError::TypeWrong),
        }
    }

    pub fn float(&self, key: &[u8]) -> Result<Float<'_>> {
        let mut r = self.key(key, Object::float);

        match r.value_mut() {
            Object::Float(_) => Ok(Float(r)),
            _ => Err(RetrievalError::TypeWrong),
        }
    }

    pub fn list(&self, key: &[u8]) -> Result<List<'_>> {
        let mut r = self.key(key, Object::list);

        match r.value_mut() {
            Object::List(_) => Ok(List(r)),
            _ => Err(RetrievalError::TypeWrong),
        }
    }

    pub fn map(&self, key: &[u8]) -> Result<Map<'_>> {
        let mut r = self.key(key, Object::map);

        match r.value_mut() {
            Object::Map(_) => Ok(Map(r)),
            _ => Err(RetrievalError::TypeWrong),
        }
    }

    pub fn set(&self, key: &[u8]) -> Result<Set<'_>> {
        let mut r = self.key(key, Object::set);

        match r.value_mut() {
            Object::Set(_) => Ok(Set(r)),
            _ => Err(RetrievalError::TypeWrong),
        }
    }

    pub fn str(&self, key: &[u8]) -> Result<Str> {
        let mut r = self.key(key, Object::string);

        match r.value_mut() {
            Object::String(_) => Ok(Str(r)),
            _ => Err(RetrievalError::TypeWrong),
        }
    }
}
