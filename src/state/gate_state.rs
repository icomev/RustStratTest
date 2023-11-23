use std::collections::HashMap;
use parking_lot::Mutex;

use crate::gate::gate_helper::ContractDetails;


const INVESTMENT_CASH: f64 = 18500.0;

pub struct GateState {
    contracts: Mutex<HashMap<String, ContractDetails>>,
}

impl GateState {
    pub fn new() -> Self {
        GateState {
            contracts: Mutex::new(HashMap::new()),
        }
    }

    pub fn update_contracts(&self, new_contracts: HashMap<String, ContractDetails>) {
        let mut contracts = self.contracts.lock();
        *contracts = new_contracts;
    }

    pub fn get_all_contracts(&self) -> HashMap<String, ContractDetails> {
        let contracts = self.contracts.lock();
        contracts.clone() // Clone the HashMap to use outside of the lock
    }

    pub async fn get_gate_contract_details(&self, symbol: &str) -> Option<(f64, i64)> {
        let contracts = self.contracts.lock();
        contracts.get(symbol).map(|details| (details.adjusted_price, details.calculated_amount))
    }

    pub async fn print_all_contracts(&self) {
        let contracts = self.contracts.lock();
        for (symbol, details) in contracts.iter() {
            println!("Symbol: {}, Details: {:?}", symbol, details);
        }
    }
    pub async fn print_specific_contract(&self, symbol: &str) {
        let contracts = self.contracts.lock();
        if let Some(contract) = contracts.get(symbol) {
            println!("{:?}", contract);
        } else {
            println!("Contract for symbol {} not found", symbol);
        }
    }

    pub async fn update_contract_details(&self, symbol: &str, last_price: f64) {
        let mut contracts = self.contracts.lock();
        if let Some(contract_details) = contracts.get_mut(symbol) {
            // New method to adjust price based on decimal places
            let last_price_str = format!("{:.10}", last_price);
            let decimal_places = last_price_str.split('.').nth(1).map_or(0, |fraction| fraction.len());
            let multiplier = 10f64.powi(decimal_places as i32);
            let adjusted_price = (last_price * 1.05 * multiplier).round() / multiplier;
    
            // Update the last_prices queue
            if contract_details.last_prices.len() >= 4 {
                contract_details.last_prices.pop_front();
            }
            contract_details.last_prices.push_back(adjusted_price);
    
            // Calculate the desired amount
            let desired_amount = INVESTMENT_CASH / (adjusted_price * contract_details.multiplier);
            let calculated_amount = desired_amount.min(contract_details.max_order_qty);
            let amount_as_integer = calculated_amount.round() as i64;
    
            // Update the amount_as_integer_history queue
            if contract_details.amount_as_integer_history.len() >= 4 {
                contract_details.amount_as_integer_history.pop_front();
            }
            contract_details.amount_as_integer_history.push_back(amount_as_integer);
    
            // Update other details
            contract_details.calculated_amount = amount_as_integer;  // Assign rounded integer value
            contract_details.adjusted_price = adjusted_price;
        } else {
            // If the symbol does not exist, you can choose to add it or log a warning
            eprintln!("Symbol not found in contract details: {}", symbol);
        }
    }


}




/*
    pub async fn update_contract_details(&self, symbol: &str, last_price: f64) {
        let mut contracts = self.contracts.lock();
        if let Some(contract_details) = contracts.get_mut(symbol) {
            let adjusted_price = (last_price * 1.05 * 100.0).round() / 100.0;

            // Update the last_prices queue
            if contract_details.last_prices.len() >= 4 {
                contract_details.last_prices.pop_front();
            }
            contract_details.last_prices.push_back(adjusted_price);

            // Calculate the desired amount
            let desired_amount = INVESTMENT_CASH / (adjusted_price * contract_details.multiplier);
            let calculated_amount = desired_amount.min(contract_details.max_order_qty);
            let amount_as_integer = calculated_amount.round() as i64;

            // Update the amount_as_integer_history queue
            if contract_details.amount_as_integer_history.len() >= 4 {
                contract_details.amount_as_integer_history.pop_front();
            }
            contract_details.amount_as_integer_history.push_back(amount_as_integer);

            // Update other details
            contract_details.calculated_amount = amount_as_integer;
            contract_details.adjusted_price = adjusted_price;
        } else {
            // If the symbol does not exist, you can choose to add it or log a warning
            eprintln!("Symbol not found in contract details: {}", symbol);
        }

*/