use serde::Deserialize;

pub trait Database: Send {
    fn get_origin(&self, key: &str) -> Origin;
}

#[derive(Deserialize, Debug)]
pub struct Origin {
    /// The origin host address
    pub host: String,
    /// The origin host port
    pub port: u16,
}
