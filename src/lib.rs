use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub api_id: i32,
    pub api_hash: String,
    pub bot_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RespResult {
    pub result: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CurrencyInfo {
    pub description: String,
    pub code: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Symbols {
    pub symbols: HashMap<String, CurrencyInfo>,
}

pub mod utils {
    pub fn load_config() -> Option<super::Config> {
        match super::File::open("config.yml") {
            Ok(context) => {
                return Some(serde_yaml::from_reader(context).expect("Could not read values."))
            }
            Err(_) => None,
        }
    }
}
