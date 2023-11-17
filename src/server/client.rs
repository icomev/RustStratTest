use futures_util::stream::StreamExt;
use futures::prelude::*;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use warp::Filter;
use tokio::sync::broadcast;

use warp::ws::{Message, WebSocket};

pub async fn setup_warp_server(processed_tx: broadcast::Sender<WsMessage>) {
    let ws_route = warp::path("websocket")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            println!("Client attempting to connect...");
            let processed_tx = processed_tx.clone();
            ws.on_upgrade(move |socket| {
                println!("WebSocket connection established with client.");
                let processed_rx = processed_tx.subscribe();
                client_connection(socket, processed_rx)
            })
        });
    warp::serve(ws_route).run(([0, 0, 0, 0], 9000)).await;
    //warp::serve(ws_route).run(([127, 0, 0, 1], 3000)).await;
    println!("Warp server running on port 9000.");
}

async fn client_connection(ws: WebSocket, mut processed_rx: broadcast::Receiver<WsMessage>) {
    let (mut client_ws_sender, mut client_ws_receiver) = ws.split();

    // Process incoming messages from client in a separate task
    let client_task = tokio::spawn(async move {
        while let Some(result) = client_ws_receiver.next().await {
            match result {
                Ok(_) => {

                },
                Err(e) => {
                    eprintln!("Error, client disconnected: {}", e);
                    // Error indicates an issue with the connection
                    break;
                }
            }
        }
    });


    // Process outgoing messages to client
    while let Ok(message) = processed_rx.recv().await {
        let forward_message = match message {
            WsMessage::Text(text) => Message::text(text),
            // Handle other message types if necessary
            _ => continue,
        };

        if client_ws_sender.send(forward_message).await.is_err() {
            eprintln!("Error sending message to client");
            break;
        }
    }

    let _ = client_task.await;
}

/*
async fn client_connection(ws: WebSocket, mut processed_rx: broadcast::Receiver<WsMessage>) {
    let (mut client_ws_sender, _) = ws.split();

    while let Ok(message) = processed_rx.recv().await {
        let forward_message = match message {
            WsMessage::Text(text) => Message::text(text),
            // Handle other message types if necessary
            _ => continue,
        };

        if client_ws_sender.send(forward_message).await.is_err() {
            eprintln!("Error sending message to client");
            break;
        }
    }
} */


/*

async fn client_connection(ws: WebSocket, mut processed_rx: broadcast::Receiver<WsMessage>) {
    let (mut client_ws_sender, mut client_ws_receiver) = ws.split();

    // Process incoming messages from client in a separate task
    let client_task = tokio::spawn(async move {
        while let Some(result) = client_ws_receiver.next().await {
            match result {
                Ok(message) => {
                    // Handle different types of messages here
                    match message {
                        warp::ws::Message::Close(_) => {
                            println!("Client disconnected");
                            return;
                        },
                        _ => {
                            // Handle other message types (e.g., text, binary)
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error receiving message from client: {}", e);
                    return;
                }
            }
        }
    });

    // Process outgoing messages to client
    while let Ok(message) = processed_rx.recv().await {
        let forward_message = match message {
            WsMessage::Text(text) => warp::ws::Message::text(text),
            // Handle other message types if necessary
            _ => continue,
        };

        if client_ws_sender.send(forward_message).await.is_err() {
            eprintln!("Error sending message to client");
            break;
        }
    }

    // Await the client task to handle any cleanup or finalization
    let _ = client_task.await;
}*/