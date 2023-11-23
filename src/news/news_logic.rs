use tokio_tungstenite::tungstenite::Error as WsError;
use url::Url;
use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use tokio::sync::broadcast;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use reqwest::Client;


use crate::news::news_helper::extract;
//use crate::news::news_helper::{ cool_off_function, extract };
use crate::news::process_logic::execute;
use crate::state::gate_state::GateState;
use crate::state::kucoin_state::KucoinState;

//use crate::news::handle::execute;
    
pub async fn connect_to_websocket(
    url: &Url,
    processed_tx: broadcast::Sender<String>,
    kucoin_client: &Arc<Client>,
    kucoin_state: &Arc<KucoinState>,
    gate_client: &Arc<Client>,
    gate_state: &Arc<GateState>
) -> Result<(), WsError> {
    let (mut ws_stream, _) = connect_async(url).await?;
    println!("Connected to {}", url);

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(WsMessage::Text(msg)) => {
                if let Some(token) = extract(&msg).await {
                    let start_time = std::time::Instant::now(); // Start timing

                    // Spawn a task for broadcasting the message
                    let broadcast_tx = processed_tx.clone();
                    let broadcast_token = token.clone();
                    tokio::task::spawn(async move {
                        if let Err(_e) = broadcast_tx.send(broadcast_token) {
                            eprintln!("Failed to broadcast message");
                        }
                    });
                    let gate_clone = gate_client.clone();
                    let gate_state_clone = gate_state.clone();
                    let kucoin_clone = kucoin_client.clone();
                    let kucoin_state_clone = kucoin_state.clone();

                    tokio::task::spawn(async move {
                        execute(gate_clone, gate_state_clone, kucoin_clone, kucoin_state_clone, token).await;
                    });

                    let end_time = std::time::Instant::now(); // End timing
                    let duration = end_time.duration_since(start_time);
                    println!("Time to process and broadcast message: {:?}", duration);
                }
            },
            Err(e) => {
                eprintln!("WebSocket closed: {}", e);
                break;
            },
            _ => {}
        }
    }

    println!("Disconnected from WebSocket.");
    Ok(())
}




/*

/*                let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();
                println!("Timestamp: {} ms, msg : {}", timestamp, msg) */*/