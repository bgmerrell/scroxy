pub type JSONStringToValue = ::serde_json::Map<String, ::serde_json::Value>;

pub trait KVStore {
    fn get(&self, key: &str) -> JSONStringToValue;
}
