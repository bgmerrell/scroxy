use std::path::PathBuf;

use ::serde_json::json;

use ::scroxy_data::{Database, Origin};
use ::scroxy_kvstore::{JSONStringToValue, KVStore};

pub struct CDB {}

impl CDB {
    // TODO: Use cdb_path to open actual CDB file(s)
    pub fn new(_cdb_path: PathBuf) -> CDB {
        CDB {}
    }
}

impl KVStore for CDB {
    fn get(&self, _key: &str) -> JSONStringToValue {
        // TODO: Replace this mocked response with a real implementation
        let value = json!(r#"{"host": "127.0.0.1", "port": 8000}"#);
        let mut map = ::serde_json::Map::new();
        map.insert("localtest".to_string(), value);
        map
    }
}

pub struct OriginDB {}

impl Database for CDB {
    fn get_origin(&self, key: &str) -> Origin {
        Origin {
            host: "127.0.0.1".to_string(),
            port: 8000,
        }
    }
}
