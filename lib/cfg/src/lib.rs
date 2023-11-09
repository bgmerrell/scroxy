use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Origin {
    /// The origin host address
    host: String,
    /// The origin host port
    port: u16,
}
