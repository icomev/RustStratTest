use url::Url;
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;
use std::sync::atomic::{AtomicBool, Ordering};


use crate::kucoin::kucoin_ws::{ fetch_ws_token, connect_bybit_websocket, ReconnectRequiredError};
use crate::kucoin::kucoin_helper::{ fetch_contracts, divide_symbols_into_groups };
use crate::state::kucoin_state::KucoinState;

pub async fn run_kucoin() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(Client::new());
    let bybit_state = Arc::new(KucoinState::new());

    let symbol_info_map = match fetch_contracts(client.clone()).await {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Error fetching contracts: {:?}", e);
            return Err(Box::<dyn std::error::Error>::from(format!("{:?}", e)));
        }
    };

    bybit_state.update_bybit(symbol_info_map);

    let all_symbols: Vec<String> = bybit_state.get_all_contracts().keys().cloned().collect();
    let symbol_groups = divide_symbols_into_groups(all_symbols, 4);

    for (index, symbols) in symbol_groups.into_iter().enumerate() {
        let state_clone = bybit_state.clone();
        let client_clone = client.clone();
        let should_continue = Arc::new(AtomicBool::new(true)); // Shared control flag for each connection

        tokio::spawn(async move {
            let token = match fetch_ws_token(&client_clone).await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error fetching WebSocket token for connection {}: {:?}", index, e);
                    return;
                }
            };

            let connect_id = Uuid::new_v4().to_string() + &index.to_string();
            let url = format!("wss://ws-api-futures.kucoin.com?token={}&connectId={}", token, connect_id);
            let parsed_url = match Url::parse(&url) {
                Ok(url) => url,
                Err(e) => {
                    eprintln!("Error parsing URL for connection {}: {:?}", index, e);
                    return;
                }
            };

            loop {
                should_continue.store(true, Ordering::SeqCst); // Reset control flag before each connection attempt

                match connect_bybit_websocket(&parsed_url, state_clone.clone(), symbols.clone(), should_continue.clone()).await {
                    Ok(()) => println!("WebSocket {} connected", index),
                    Err(e) => {
                        if e.downcast_ref::<ReconnectRequiredError>().is_some() {
                            eprintln!("Reconnection required for Kucoin {}", index);
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        } else {
                            eprintln!("Error in WebSocket at Kucoin {}: {:?}", index, e);
                            break;
                        }
                    },
                }
            }
        });
    }

    futures::future::pending::<()>().await;
    Ok(())
}

