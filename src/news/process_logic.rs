use reqwest::Client;
use std::time::Duration;
use std::sync::Arc;

use crate::state::gate_state::GateState;
use crate::state::kucoin_state::KucoinState;
use crate::kucoin::kucoin_orders::{ execute_kucoin_order, execute_sell_order };
use crate::gate::gate_orders::{ execute_gate_buy, execute_gate_sell };


const SELL_DELAYS: [f32; 5] = [0.8, 1.0, 1.4, 1.8, 2.4];
//const SELL_DELAYS: [f32; 5] = [5.8, 10.0, 15.4, 20.8, 25.4];


    pub async fn gate_calc(gate_client: &Client, shared_state: &Arc<GateState>, value: String) {
        // Fetching specific contract details and checking in the same line
        if let Some((adjusted_price, amount_as_integer)) = shared_state.get_gate_contract_details(&format!("{}_USDT", value)).await {
            // Execute the buy order once
        let bought_amount = execute_gate_buy(&gate_client, &value, adjusted_price, amount_as_integer).await;
        println!("Successfully bought size gate {}", bought_amount);

        // Calculate the amount to sell in each order and make it negative
        let amount_per_sell = -(bought_amount / 5.0).ceil();
        let mut tasks = vec![];
        
        for i in 0..5 {
            let gate_client_clone = gate_client.clone();
            let value_clone = format!("{}_USDT", value);
            let sell_delay = SELL_DELAYS[i];

            let task = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs_f32(sell_delay)).await;
                execute_gate_sell(&gate_client_clone, &value_clone, amount_per_sell).await;
            });

            tasks.push(task);
        }

        let _ = futures::future::join_all(tasks).await;
    } else {
        eprintln!("No data available for gate {}", value);
    }
}



pub async fn kucoin_calc(kucoin_client: &Client, shared_state: &Arc<KucoinState>, value: String,) {
    let symbol = format!("{}USDTM", value);

    if let Some((adjusted_price, num_contracts)) = shared_state.get_specific_contract_details(&symbol).await {

//    if let Some((adjusted_price, num_contracts)) = shared_state.get_specific_contract_details(&format!("{}USDTM", value)).await {

        let bought_amount = execute_kucoin_order(&kucoin_client, &value, adjusted_price, num_contracts)
        .await
            .unwrap_or_else(|err| {
                eprintln!("Failed to execute buy order: {}", err);
                0
            });

            println!("bought amount kucoin {}", bought_amount);
            let amount_per_sell = ((bought_amount as f64) / 5.0).ceil();
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
        eprintln!("No data available for kucoin {}", symbol);
    }
}


pub async fn execute(gate_clone: Arc<Client>, gate_state: Arc<GateState>, kucoin_clone: Arc<Client>, kucoin_state: Arc<KucoinState>,  token: String) {

    let token_gate = token.clone();

    let token_kucoin = token.clone();

    tokio::task::spawn(async move {
        gate_calc(&*gate_clone, &gate_state, token_gate).await;
    });

    tokio::task::spawn(async move {
        kucoin_calc(&kucoin_clone, &kucoin_state, token_kucoin).await;
    });

}
 


