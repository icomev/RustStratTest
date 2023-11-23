use std::collections::HashMap;
use parking_lot::Mutex;

use crate::kucoin::kucoin_helper::Contract;
const INVESTMENT_CASH: f64 = 10000.0;


pub struct KucoinState {
    pub contracts: Mutex<HashMap<String, Contract>>,
}

fn decimal_places(value: f64) -> u32 {
    format!("{}", value)
        .split('.')
        .nth(1)
        .map_or(0, |fraction| fraction.chars().take_while(|&c| c != 'e').count()) as u32
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

    pub fn get_contract_multiplier(&self, symbol: &str) -> f64 {
        // Lock the mutex to access the contracts
        let contracts = self.contracts.lock();
        contracts.get(symbol).map_or(1.0, |contract| contract.multiplier)
    }



    pub async fn get_specific_contract_details(&self, symbol: &str) -> Option<(f64, i64)> {
        let contracts = self.contracts.lock();
        if let Some(details) = contracts.get(symbol) {
            let adjusted_price = match details.adjusted_prices.as_ref() {
                Some(prices) if !prices.is_empty() => *prices.last().unwrap(),
                _ => {
                    println!("No adjusted prices available for {}", symbol);
                    return None;
                },
            };
            let num_contracts = match details.num_contracts.as_ref() {
                Some(contracts) if !contracts.is_empty() => *contracts.last().unwrap() as i64,
                _ => {
                    println!("No number of contracts available for {}", symbol);
                    return None;
                },
            };
            Some((adjusted_price, num_contracts))
        } else {
            println!("No contract found for {}", symbol);
            None
        }
    }

    pub async fn print_contract_details(&self, symbol: &str) {
        let contracts = self.contracts.lock();
        if let Some(symbol_info) = contracts.get(symbol) {
            println!("Details for symbol '{}':", symbol);
            if let Some(adjusted_prices) = &symbol_info.adjusted_prices {
                println!("Adjusted Prices: {:?}", adjusted_prices);
            } else {
                println!("No adjusted prices available.");
            }
            if let Some(num_contracts) = &symbol_info.num_contracts {
                println!("Number of Contracts: {:?}", num_contracts);
            } else {
                println!("No number of contracts available.");
            }
        } else {
            println!("No data available for symbol '{}'", symbol);
        }
    }

    pub async fn update_contract_details(&self, symbol: &str, last_price: f64, contract_multiplier: f64) {
        let decimals = decimal_places(last_price); // Function to calculate decimal places
        let price_multiplier = 10f64.powi(decimals as i32);
        let adjusted_price = (last_price * 1.049 * price_multiplier).round() / price_multiplier;
    
        // Calculate the total value represented by one contract
        let contract_value = adjusted_price * contract_multiplier;
    
        // Calculate the number of contracts you can afford
        let num_contracts = (INVESTMENT_CASH / contract_value).round(); 
        //println!("Details for symbol '{}':", num_contracts);

    
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

