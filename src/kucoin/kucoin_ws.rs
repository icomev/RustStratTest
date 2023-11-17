use serde_json::json;
use tokio_tungstenite::connect_async;
use tungstenite::protocol::Message;
use futures_util::stream::StreamExt;
use futures::SinkExt;
use url::Url;
use std::sync::Arc;
use tokio::time::{Duration, interval};
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;
use std::fmt;
use anyhow::{ Error, Result }; // Import Error from anyhow
use std::error::Error as StdError; // Rename `std::error::Error` to `StdError`
use std::sync::atomic::{AtomicBool, Ordering};


use crate::kucoin::kucoin_helper::handle_ticker_update;
use crate::state::kucoin_state::KucoinState;


#[derive(Debug)]
pub struct ReconnectRequiredError;

impl fmt::Display for ReconnectRequiredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reconnection required")
    }
}

impl StdError for ReconnectRequiredError {}


pub async fn fetch_ws_token(client: &Client) -> Result<String> {
    let response = client
        .post("https://api-futures.kucoin.com/api/v1/bullet-public")
        .send()
        .await?;
    
    let raw_text = response.text().await?;

    let value: Value = serde_json::from_str(&raw_text)?;

    if let Some(token) = value["data"]["token"].as_str() {
        Ok(token.to_string())
    } else {
        // Use anyhow::Error for error handling
        Err(anyhow::Error::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Token not found in the response: {}", raw_text),
        )))
    }
}


pub async fn connect_bybit_websocket(
    url: &Url,
    shared_state: Arc<KucoinState>,
    symbols_to_subscribe: Vec<String>,
    should_continue: Arc<AtomicBool>, // Add this parameter
) -> Result<(), Error> {
    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to wss://ws-api-futures.kucoin.com");

    let (mut write, mut read) = ws_stream.split(); // Changed _read to read
    let mut ping_interval = interval(Duration::from_secs(18));

    // Subscribe only to the symbols in symbols_to_subscribe
    for symbol in symbols_to_subscribe {
        let message_id = Uuid::new_v4().to_string(); // Generate a unique ID for each message

        let subscribe_message = json!({
            "id": message_id,
            "type": "subscribe",
            "topic": format!("/contractMarket/tickerV2:{}", symbol),
            "privateChannel": false,
            "response": true
        }).to_string();

        write.send(Message::Text(subscribe_message)).await.expect("Failed to send message");
    }

    let ping_should_continue = Arc::clone(&should_continue);
    let ping_task = tokio::spawn(async move {
        loop {
            ping_interval.tick().await; // Wait for the next interval tick

            let ping_message = {
                let timestamp = Utc::now().timestamp_millis(); // Get the current timestamp in milliseconds
                json!({
                    "id": timestamp.to_string(), // Use the timestamp as the unique ID
                    "type": "ping"
                }).to_string()
            };
    
            if write.send(Message::Text(ping_message)).await.is_err() {
                eprintln!("Failed to send ping message kucoin");
                ping_should_continue.store(false, Ordering::SeqCst); // Signal the other task to stop
                break;
            }
        }
    });

        while let Some(message) = read.next().await {

            match message {
                Ok(text_message) => {
                    if let Message::Text(message) = text_message {
                        
                        let shared_state_clone = Arc::clone(&shared_state);
                        let handle = tokio::task::spawn(async move {
                            handle_ticker_update(&message, shared_state_clone).await;
                        });
                        
                        if let Err(e) = handle.await {
                            let now = Utc::now();
                            eprintln!("{} - Task panicked at Kucoin::: {:?}", now.to_rfc3339(), e);
                            break;

                            
                        }
                    }
                }
                Err(e) => {
                    let now = Utc::now();
                    eprintln!("{} - Reading message kucoin fail: {:?}", now.to_rfc3339(), e);
                }
            }
        }

    tokio::select! {
        _ = ping_task => {
            eprintln!("Ping task finished unexpectedly Kucoin.");
            return Err(anyhow::Error::new(ReconnectRequiredError));
        },
    }
}



/*


pub async fn connect_bybit_websocket(
    url: &Url,
    shared_state: Arc<KucoinState>,
    symbols_to_subscribe: Vec<String>,
    should_continue: Arc<AtomicBool>, // Add this parameter
) -> Result<(), Error> {
    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to wss://ws-api-futures.kucoin.com");

    let (mut write, mut read) = ws_stream.split(); // Changed _read to read
    let mut ping_interval = interval(Duration::from_secs(18));

    // Subscribe only to the symbols in symbols_to_subscribe
    for symbol in symbols_to_subscribe {
        let message_id = Uuid::new_v4().to_string(); // Generate a unique ID for each message

        let subscribe_message = json!({
            "id": message_id,
            "type": "subscribe",
            "topic": format!("/contractMarket/tickerV2:{}", symbol),
            "privateChannel": false,
            "response": true
        }).to_string();

        write.send(Message::Text(subscribe_message)).await.expect("Failed to send message");
    }

    let ping_should_continue = Arc::clone(&should_continue);
    let ping_task = tokio::spawn(async move {
        loop {
            ping_interval.tick().await; // Wait for the next interval tick

            let ping_message = {
                let timestamp = Utc::now().timestamp_millis(); // Get the current timestamp in milliseconds
                json!({
                    "id": timestamp.to_string(), // Use the timestamp as the unique ID
                    "type": "ping"
                }).to_string()
            };
    
            if write.send(Message::Text(ping_message)).await.is_err() {
                eprintln!("Failed to send ping message kucoin");
                ping_should_continue.store(false, Ordering::SeqCst); // Signal the other task to stop
                break;
            }
        }
    });

    let read_should_continue = Arc::clone(&should_continue);
    let read_task = tokio::spawn(async move {
        while let Some(message) = read.next().await {

            match message {
                Ok(text_message) => {
                    if let Message::Text(message) = text_message {
                        
                        let shared_state_clone = Arc::clone(&shared_state);
                        let handle = tokio::task::spawn(async move {
                            handle_ticker_update(&message, shared_state_clone).await;
                        });
                        
                        if let Err(e) = handle.await {
                            let now = Utc::now();
                            eprintln!("{} - Task panicked at Kucoin::: {:?}", now.to_rfc3339(), e);
                            read_should_continue.store(false, Ordering::SeqCst); // Signal the other task to stop
                            break;

                            
                        }
                    }
                }
                Err(e) => {
                    let now = Utc::now();
                    eprintln!("{} - Read task: error receiving message from Kucoin:: {:?}", now.to_rfc3339(), e);
                }
            }
        }
    });

    tokio::select! {
        _ = read_task => {
            let now = Utc::now();
            eprintln!("{} - Read task finished unexpectedly Kucoin", now.to_rfc3339());
            return Err(anyhow::Error::new(ReconnectRequiredError));
        },
        _ = ping_task => {
            eprintln!("Ping task finished unexpectedly Kucoin.");
            return Err(anyhow::Error::new(ReconnectRequiredError));
        },
    }
}*/