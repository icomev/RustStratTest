use std::collections::HashMap;
use parking_lot::Mutex;

use crate::kucoin::kucoin_helper::Contract;
const INVESTMENT_CASH: f64 = 21000.0;


pub struct KucoinState {
    contracts: Mutex<HashMap<String, Contract>>,
}

impl KucoinState {
    pub fn new() -> Self {
        KucoinState {
            contracts: Mutex::new(HashMap::new()),
        }
    }

    pub fn update_bybit(&self, new_contracts: HashMap<String, Contract>) {
        let mut contracts = self.contracts.lock();
        *contracts = new_contracts;
    }

    pub fn get_all_contracts(&self) -> HashMap<String, Contract> {
        let contracts = self.contracts.lock();
        contracts.clone() // Clone the HashMap to use outside of the lock
    }

    pub async fn get_specific_contract_details(&self, symbol: &str) -> Option<(f64, i64)> {
        let contracts = self.contracts.lock();
        contracts.get(symbol).and_then(|details| {
            let adjusted_price = details.adjusted_prices.as_ref()?.last()?.to_owned();
            let num_contracts = details.num_contracts.as_ref()?.last()?.to_owned() as i64;
            Some((adjusted_price, num_contracts))
        })
    }
    pub async fn update_contract_details(&self, symbol: &str, last_price: f64) {
        let decimal_places = format!("{:e}", last_price).split('.').nth(1).map_or(0, |fraction| fraction.len());
        let multiplier = 10f64.powi(decimal_places as i32);
        let adjusted_price = (last_price * 1.05 * multiplier).round() / multiplier;
        let num_contracts = (INVESTMENT_CASH / adjusted_price).round();
    
        let mut contracts = self.contracts.lock();
        if let Some(symbol_info) = contracts.get_mut(symbol) {
            // Ensure adjusted_prices and num_contracts are initialized if they're None
            if symbol_info.adjusted_prices.is_none() {
                symbol_info.adjusted_prices = Some(Vec::new());
            }
            if symbol_info.num_contracts.is_none() {
                symbol_info.num_contracts = Some(Vec::new());
            }
    
            // Now it's safe to unwrap because we just ensured they are not None
            let adjusted_prices = symbol_info.adjusted_prices.as_mut().unwrap();
            let num_contracts_vec = symbol_info.num_contracts.as_mut().unwrap();
    
            if adjusted_prices.len() >= 4 { adjusted_prices.remove(0); }
            if num_contracts_vec.len() >= 4 { num_contracts_vec.remove(0); }
    
            adjusted_prices.push(adjusted_price);
            num_contracts_vec.push(num_contracts);
        }
    }

}

