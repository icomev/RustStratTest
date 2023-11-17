use url::Url;
use reqwest::Client;
use std::sync::Arc;
use chrono::Utc;

use crate::gate::gate_helper::get_futures_contracts;
use crate::gate::gate_ws::connect_gate_websocket;

use crate::state::app_state::SharedState;



pub async fn run_gate() {

    let gate_lesser = Arc::new(Client::new());
    let shared_state = Arc::new(SharedState::new());


    let gate_websocket = tokio::spawn(async move {
        match get_futures_contracts(gate_lesser.clone()).await {
        Ok(usdt_contracts) => {
            shared_state.update_contracts(usdt_contracts);

            let ws_url = Url::parse("wss://fx-ws.gateio.ws/v4/ws/usdt").expect("Failed to parse WebSocket URL");
            let shared_state_for_websocket = shared_state.clone();

                loop {
                    match connect_gate_websocket(&ws_url, shared_state_for_websocket.clone()).await {
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
    });
    let _ = gate_websocket.await;

}








/*

    /*
        l
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Sleep for 1 hour
            let now = Utc::now();
            println!("time {}...", now);
                shared_state.print_specific_contract("DOGE_USDT").await;           
           
              */*/