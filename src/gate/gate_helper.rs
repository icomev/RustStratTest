use serde::Deserialize;
use std::sync::Arc;
use reqwest::Client;
use std::collections::HashSet;
use std::collections::VecDeque;
use reqwest::Error;
use std::collections::HashMap;
use serde_json::Value;
use serde::Serialize;


use crate::state::app_state::SharedState;


#[derive(Deserialize, Debug)]
pub struct FutureContract {
    pub name: String,
    pub quanto_multiplier: String,
    pub order_size_max: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct ContractDetails {
    pub max_order_qty: f64,
    pub multiplier: f64,
    pub last_prices: VecDeque<f64>, 
    pub adjusted_price: f64, 
    pub amount_as_integer_history: VecDeque<i64>, 
    pub calculated_amount: i64,
}


//const INVESTMENT_CASH: f64 = 800.0;


pub async fn handle_ticker_update(message: &str, shared_state: Arc<SharedState>) {
    if message.is_empty() {
        return;
    }
    let v: Value = match serde_json::from_str(message) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Failed to parse JSON: {}. The message was: {}", err, message);
            return;
        }
    };

    if v["event"] != "update" || v["channel"] != "futures.tickers" {
        return;
    }

    let results = match v["result"].as_array() {
        Some(results) => results,
        None => return,
    };

    for result in results {
        let contract_obj = match result.as_object() {
            Some(obj) => obj,
            None => continue,
        };

        let symbol = match contract_obj.get("contract").and_then(Value::as_str) {
            Some(contract) => contract.to_string(), 
            None => continue,
        };

        let last_price_str = match contract_obj.get("last").and_then(Value::as_str) {
            Some(last) => last,
            None => continue,
        };

        let last_price: f64 = match last_price_str.parse() {
            Ok(price) => price,
            Err(_) => continue,
        };

        // Call a method on shared_state to update the contract details
        shared_state.update_contract_details(&symbol, last_price).await;
    }
}




pub async fn get_futures_contracts(less_important_client: Arc<Client>) -> Result<HashMap<String, ContractDetails>, Error> {
    let resp = less_important_client.get("https://fx-api.gateio.ws/api/v4/futures/usdt/contracts").send().await?;
    
    let contracts: Vec<FutureContract> = resp.json().await?;

    let blacklist: HashSet<String> = [

    ]
    .iter()
    .cloned()
    .collect();

    let mut usdt_contracts = HashMap::new();
    
    for contract in contracts {
        if contract.name.ends_with("_USDT") && !blacklist.contains(&contract.name) {
            let multiplier_as_f64: f64 = match contract.quanto_multiplier.parse() {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Failed to parse quanto_multiplier for {}: {}", contract.name, e);
                    continue;  // Skip this contract and move on to the next
                }
            };
    
            let details = ContractDetails {
                multiplier: multiplier_as_f64,
                max_order_qty: contract.order_size_max as f64,
                last_prices: VecDeque::from(vec![0.0]), // Initialize with a VecDeque
                adjusted_price: 0.0,
                amount_as_integer_history: VecDeque::from(vec![0]), // Initialize with a VecDeque
                calculated_amount: 0, // You should initialize this value appropriately
            };
            usdt_contracts.insert(contract.name.clone(), details);
        }
    }

    Ok(usdt_contracts)
}






