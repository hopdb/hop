use hop_engine::state::{KeyType, Value};

pub fn key_type_name(key_type: KeyType) -> &'static str {
    match key_type {
        KeyType::Boolean => "bool",
        KeyType::Bytes => "bytes",
        KeyType::Float => "float",
        KeyType::Integer => "int",
        KeyType::List => "list",
        KeyType::Map => "map",
        KeyType::Set => "set",
        KeyType::String => "str",
    }
}

pub fn list(items: Vec<Vec<u8>>) -> String {
    let mut output = String::new();

    for item in items {
        output.push_str(String::from_utf8_lossy(&item).as_ref());
        output.push('\n');
    }

    output.truncate(output.len() - 2);

    output
}

pub fn map<T: IntoIterator<Item = (Vec<u8>, Vec<u8>)>>(pairs: T) -> String {
    let mut output = String::new();

    for (k, v) in pairs {
        output.push_str(String::from_utf8_lossy(&k).as_ref());
        output.push('=');
        output.push_str(String::from_utf8_lossy(&v).as_ref());
        output.push('\n');
    }

    output.truncate(output.len() - 2);

    output
}

pub fn value(value: Value) -> String {
    match value {
        Value::Boolean(boolean) => boolean.to_string(),
        Value::Bytes(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Value::Float(float) => float.to_string(),
        Value::Integer(int) => int.to_string(),
        Value::List(value_list) => list(value_list),
        Value::Map(value_map) => map(value_map.into_iter()),
        Value::Set(set) => list(set.into_iter().collect()),
        Value::String(string) => string,
    }
}
