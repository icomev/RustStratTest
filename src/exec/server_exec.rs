use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use url::Url;
use tokio::sync::broadcast;


use crate::server::client::setup_warp_server ;
use crate::news::news_logic::connect_to_websocket;




pub async fn run_server() {
    let bybit_url_str = "wss://news.treeofalpha.com/ws";
    let bybit_url = Url::parse(bybit_url_str).expect("Invalid WebSocket URL");

    let (processed_tx, _processed_rx) = broadcast::channel::<WsMessage>(500);

    // Clone the sender for use in the server_task
    let processed_tx_clone = processed_tx.clone();

    let server_task = tokio::spawn(async move {
        loop {
            match connect_to_websocket(&bybit_url, processed_tx_clone.clone()).await {
                Ok(_) => println!("Server connection closed gracefully."),
                Err(e) => println!("Server connection error: {}. Attempting to reconnect...", e),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });

    let warp_task = tokio::spawn(async move {
        println!("Starting Warp server...");
    
        loop {
            setup_warp_server(processed_tx.clone()).await;
            println!("Server connection error. Attempting to reconnect...");
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });

 
    let _ = tokio::try_join!(server_task, warp_task);
    println!("All tasks completed.");
}