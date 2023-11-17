use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use reqwest::Client;
use serde_json::Value;

use crate::state::kucoin_state::KucoinState;

#[derive(Deserialize, Serialize, Clone)]
pub struct Contract {
    symbol: String,
    multiplier: f64,
    pub adjusted_prices: Option<Vec<f64>>, // Mark as optional
    pub num_contracts: Option<Vec<f64>>,   // Mark as optional
}

#[derive(Deserialize, Serialize)]
struct Response {
    data: Vec<Contract>,
    // Add other fields as necessary
}

#[derive(Debug)]
pub enum FetchError {
    HttpRequestError(reqwest::Error),
    RequestFailed(String),
    ParsingError(serde_json::Error),
}


pub async fn handle_ticker_update(message: &str, shared_state: Arc<KucoinState>) {

    let v: Value = match serde_json::from_str(message) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Failed to parse JSON: {}. The message was: {}", err, message);
            return;
        }
    };
   

    if let Some(symbol) = v["data"]["symbol"].as_str() {
        if let Some(last_price_str) = v["data"]["bestBidPrice"].as_str() {

            if let Ok(last_price) = last_price_str.parse::<f64>() {
                shared_state.update_contract_details(symbol, last_price).await;
            } else {
                eprintln!("Failed to parse 'lastPrice' field as a floating-point number from string: {}", last_price_str);
            }
        }
    }
}

pub async fn fetch_contracts(client: Arc<Client>) -> Result<HashMap<String, Contract>, FetchError> {
    let url = "https://api-futures.kucoin.com/api/v1/contracts/active";

    let response = reqwest::get(url).await.map_err(FetchError::HttpRequestError)?;
    
    if response.status().is_success() {
        let resp_body = response.text().await.map_err(FetchError::HttpRequestError)?;
        let parsed_response: Response = serde_json::from_str(&resp_body).map_err(FetchError::ParsingError)?;

        let mut contracts_map: HashMap<String, Contract> = HashMap::new();
        for contract in parsed_response.data {
            if contract.symbol.ends_with("USDTM") {
                let mut contract_details = contract;
                contract_details.adjusted_prices = None; // Initialize as None
                contract_details.num_contracts = None;    // Initialize as None
                contracts_map.insert(contract_details.symbol.clone(), contract_details);
            }
        }
        Ok(contracts_map)

    } else {
        Err(FetchError::RequestFailed(format!("Request failed with status: {}", response.status())))
    }
}

pub fn divide_symbols_into_groups(symbols: Vec<String>, group_count: usize) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    let group_size = symbols.len() / group_count;

    for i in 0..group_count {
        let start_index = i * group_size;
        let end_index = if i == group_count - 1 { symbols.len() } else { start_index + group_size };
        groups.push(symbols[start_index..end_index].to_vec());
    }

    groups
}