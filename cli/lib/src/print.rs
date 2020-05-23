use hop_engine::state::KeyType;

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
