use std::{fmt::format, collections::HashMap};

use exchange_bot::Config;
use reqwest;
use serde::{self, Deserialize};
use async_trait::async_trait;


#[async_trait]
pub trait Exchange {
    async fn get_list(self) -> Option<Symbols>;
    async fn convert(self, from: &str, target: &str, value: &str) -> Option<ConvertResult>;
}

pub struct ExchangeClient {
    pub endpoint: String,
    
}

impl ExchangeClient {
    pub fn new(apikey: &str) -> Self {
        Self {
            endpoint: format!("http://api.exchangerate.host/|req|?{apikey}", apikey=apikey),
        }
    }
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

#[derive(Debug, Deserialize)]
pub struct ConvertResult {
    pub result: f64,
}

#[async_trait]
impl Exchange for ExchangeClient {
    async fn get_list(self) -> Option<Symbols>{
        let url: String = self.endpoint.replace("|req|", "list"); 
        match reqwest::get(&url).await {
            Ok(response) => {
                return Some(response.json::<Symbols>().await.unwrap())
            }
            Err(_) => {
                println!("Request /list error");
                None
            }
        }
    }

    async fn convert(self, from: &str, target: &str, value: &str) -> Option<ConvertResult> {
        let mut url: String = self.endpoint.replace("|req|", "convert&from"); 
        url + format!("&from={from}&to={target}&amount={amount}", from=from, target=target, amount=value).as_str();
        match reqwest::get(&url).await {
            Ok(response) => {
                return Some(response.json::<ConvertResult>().await.unwrap())
            }
            Err(_) => {
                println!("Request /convert error");
                None
            }
        }
    }
}

