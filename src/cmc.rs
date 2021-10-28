use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[derive(Debug, Serialize, Deserialize)]
pub struct Quotes(pub HashMap<String, Quote>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    pub price: f64,
    pub percent_change_7d: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Currency {
    pub name: String,
    pub symbol: String,
    pub quote: Quotes,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Name: {}, Symbol: {}, Price: {}, Change(7d): {}%",
            self.name,
            self.symbol,
            self.quote.0.get("USD").unwrap().price.to_string(),
            self.quote
                .0
                .get("USD")
                .unwrap()
                .percent_change_7d
                .to_string()
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CMCResponse {
    pub data: HashMap<String, Currency>,
}
