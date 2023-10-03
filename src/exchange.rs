use std::fmt::format;

use reqwest;
use serde;
use async_trait::async_trait;


#[async_trait]
pub trait Exchange {
    async fn get_list(self);
    async fn convert(self);
}

pub struct ExchangeClient {
    pub base_url: String,
    pub api_keys: String,
}


#[async_trait]
impl Exchange for ExchangeClient {
    async fn get_list(self) {
        let url: String = format!("http://api.exchangerate.host/list?access_key={}", self.api_keys); 
        // Request http://api.exchangerate.host/live?access_key=4bac4a0c5e53688ae5aa6703d84803c9
        // let url = format("");
        todo!()
    }

    async fn convert(self) {
        todo!()
    }
}

