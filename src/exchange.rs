use std::collections::HashMap;

use async_trait::async_trait;
use reqwest;
use serde::{self, Deserialize};

#[async_trait]
pub trait Exchange {
    async fn get_list(self) -> Option<Symbols>;
    async fn convert(self, from: &str, target: &str, value: &str) -> Option<ConvertResult>;
}

#[derive(Clone)]
pub struct ExchangeClient {
    pub endpoint: String,
}

// impl Copy for ExchangeClient {}
trait Copy {
    fn copy(&self) -> Self;
}


impl ExchangeClient {
    pub fn new(apikey: &str) -> Self {
        Self {
            endpoint: format!(
                "http://api.exchangerate.host/|req|?access_key={apikey}",
                apikey = apikey
            ),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Symbols {
    pub success: bool,
    pub terms: String,
    pub privacy: String,
    #[serde(rename="currencies")]
    pub symbols: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ConvertResult {
    pub result: f64,
}

#[async_trait]
impl Exchange for ExchangeClient {
    async fn get_list(self) -> Option<Symbols> {
        let url: String = self.endpoint.replace("|req|", "list");
        match reqwest::get(&url).await {
            Ok(response) => {
                Some(response.json::<Symbols>().await.unwrap())
            }
            Err(_) => {
                println!("Request /list error");
                None
            }
        }
    }

    async fn convert(self, from: &str, target: &str, value: &str) -> Option<ConvertResult> {
        let mut url: String = self.endpoint.replace("|req|", "convert&from");
        let url = url + format!(
            "&from={from}&to={target}&amount={amount}",
            from = from,
            target = target,
            amount = value
        )
        .as_str();
        match reqwest::get(&url).await {
            Ok(response) => return Some(response.json::<ConvertResult>().await.unwrap()),
            Err(_) => {
                println!("Request /convert error");
                None
            }
        }
    }
}
