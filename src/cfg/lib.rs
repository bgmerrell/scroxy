use serde::Deserialize;

#[dervice(Deserialize, Debug)]
pub struct Origin {
    /// The origin host address
    host: String,
    /// The origin host port
    port: u16,
}
