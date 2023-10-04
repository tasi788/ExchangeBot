use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub api_id: i32,
    pub api_hash: String,
    pub bot_token: String,
    pub api_endpoint: String,
    pub api_token: String,
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
