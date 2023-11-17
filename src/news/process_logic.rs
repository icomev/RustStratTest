use reqwest::Client;
use std::time::Duration;
use std::sync::Arc;
use tokio::task::spawn;
use futures::future::join_all;
use tokio::time::sleep;


use crate::state::app_state::get_specific_contract_details;



    pub async fn gate_calc(gate_client: &Client, value: String, shared_state: Arc<SharedState>) {
        // Fetching specific contract details and checking in the same line
        if let Some((adjusted_price, amount_as_integer)) = shared_state.get_specific_contract_details(&format!("{}_USDT", value)).await {
            // Execute the buy order once
        let bought_amount = execute_buy_order(&gate_client, &value, adjusted_price, amount_as_integer).await;
        println!("Successfully bought size gate {}", bought_amount);

        // Calculate the amount to sell in each order and make it negative
        let amount_per_sell = -(bought_amount / 5.0).round();
        let mut tasks = vec![];
        
        for i in 0..5 {
            let gate_client_clone = gate_client.clone();
            let value_clone = format!("{}_USDT", value);
            let sell_delay = SELL_DELAYS[i];

            let task = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs_f32(sell_delay)).await;
                execute_sell_order(&gate_client_clone, &value_clone, amount_per_sell).await;
            });

            tasks.push(task);
        }

        let _ = futures::future::join_all(tasks).await;
    } else {
        eprintln!("No data available for gate {}", value);
    }
}



pub async fn kucoin_calc(kucoin_client: &Client, value: String, shared_state: Arc<KucoinState>) {
    let symbol_with_suffix = format!("{}USDTM", value);

    if let Some((adjusted_price, num_contracts)) = shared_state.get_specific_contract_details(&format!("{}USDTM", value)).await {
        match execute_kucoin_order(kucoin_client, symbol_with_suffix, adjusted_price, num_contracts).await {
            Ok(bought_amount) => println!("Successfully bought size: {}", bought_amount),
            Err(e) => eprintln!("Failed to execute kucoin order: {:?}", e),
        }
        let amount_per_sell = -(bought_amount / 5.0).round();
        let mut tasks = vec![];
        
        for i in 0..5 {
            let kucoin_client_clone = kucoin_client.clone();
            let value_clone = format!("{}USDTM", value);
            let sell_delay = SELL_DELAYS[i];
    
            let task = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs_f32(sell_delay)).await;
                match execute_sell_order(&kucoin_client_clone, &value_clone, amount_per_sell).await {
                    Ok(_) => println!("Successfully executed sell order."),
                    Err(e) => eprintln!("Failed to execute sell order: {:?}", e),
                }
            });
    
            tasks.push(task);
        }
    
        let _ = futures::future::join_all(tasks).await;
    } else {
        eprintln!("No data available for gate {}", value);
    }
}


pub async fn execute(gate_clone: Arc<Client>, bybit_clone: Arc<Client>, token: String) {

    let token_gate = token.clone();

    let token_kucoin = token.clone();

    tokio::task::spawn(async move {
        gate_calc(&*gate_clone, token_gate).await;
    });

    tokio::task::spawn(async move {
        kucoin_calc(&bybit_clone, token_kucoin).await;
    });

}



