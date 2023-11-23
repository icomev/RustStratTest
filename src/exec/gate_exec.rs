use url::Url;
use reqwest::Client;
use std::sync::Arc;
use chrono::Utc;

use crate::gate::gate_helper::get_futures_contracts;
use crate::gate::gate_ws::connect_gate_websocket;
use crate::state::gate_state::GateState;

pub async fn run_gate(gate_lesser: &Arc<Client>, gate_state: Arc<GateState>) {

    //let gate_lesser = Arc::new(Client::new());
    //let shared_state = Arc::new(GateState::new());


        match get_futures_contracts(gate_lesser.clone()).await {
        Ok(usdt_contracts) => {
            gate_state.update_contracts(usdt_contracts);

            let ws_url = Url::parse("wss://fx-ws.gateio.ws/v4/ws/usdt").expect("Failed to parse WebSocket URL");
            let gate_state_clone = gate_state.clone();

                loop {
                    match connect_gate_websocket(&ws_url, gate_state_clone.clone()).await {
                        Ok(()) => {
                            let now = Utc::now();
                            eprintln!("{} - WebSocket connection closed cleanly, will attempt to reconnect to Gate...", now.to_rfc3339());
                        },
                        Err(e) => {
                            let now = Utc::now();
                            eprintln!("{} - Error with WebSocket connection Gate: {:?}", now.to_rfc3339(), e);
                        }
                    }
                    

                    // Reconnection backoff
                    eprintln!("{} - Attempting to reconnect to WebSocket at Gate...", Utc::now().to_rfc3339());
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
                

        },
    
        Err(e) => {
            eprintln!("Failed to get futures contracts: {:?}", e);
        },

    }


}

/*
    let gate_test = Arc::new(Client::new());
    let shared_state = Arc::new(SharedState::new());
    let clone = shared_state.clone();

    let test_task = tokio::spawn(async move {  // Use `move` here
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        //bybit_state.print_contract_details("DOGEUSDTM").await;
        //tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        // Assuming "DOGEUSDTM" is the value you want to test with
        let test_value = "DOGE".to_string(); // Use the exact key as stored in the state
        gate_calc(&gate_test, &clone, test_value).await;
    });
*/