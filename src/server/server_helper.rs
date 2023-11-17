
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio::sync::broadcast;



pub async fn process_message(msg: WsMessage, processed_tx: broadcast::Sender<WsMessage>) {
    if let Err(e) = processed_tx.send(msg) {
        eprintln!("Failed to send message to the local server: {:?}", e);

    }
}