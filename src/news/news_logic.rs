use tokio_tungstenite::tungstenite::Error as WsError;
use url::Url;
use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use tokio::sync::broadcast;
use std::time::{SystemTime, UNIX_EPOCH};

//use crate::news::news_helper::{ cool_off_function, extract };
//use crate::news::handle::execute;


pub async fn connect_to_websocket(
    url: &Url, 
    processed_tx: broadcast::Sender<WsMessage>
) -> Result<(), WsError> {
    let (mut ws_stream, _) = connect_async(url).await?;
    println!("Connected to {}", url);

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(WsMessage::Text(msg)) => {
                let start_time = std::time::Instant::now(); // Start timing

                let new_msg = WsMessage::Text(msg.clone()); 
                if let Err(_e) = processed_tx.send(new_msg) {
                    eprintln!("Failed to broadcast message");
                }

                let end_time = std::time::Instant::now(); // End timing
                let duration = end_time.duration_since(start_time);
                println!("Time to process and broadcast message: {:?}", duration);

/*                let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();
                println!("Timestamp: {} ms, msg : {}", timestamp, msg) */
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





