use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EODResponse {
    pub code: String,
    pub close: f64,
}
