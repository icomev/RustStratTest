use url::Url;
use tokio::sync::broadcast;
use reqwest::Client;
use std::sync::Arc;

use crate::server::client::setup_warp_server ;
use crate::news::news_logic::connect_to_websocket;
use crate::gate::gate_orders::keep_alive_transaction;
use crate::kucoin::kucoin_orders::keep_alive_kucoin;
use crate::state::kucoin_state::KucoinState;
use crate::state::gate_state::GateState;




pub async fn run_server(gate_client: &Arc<Client>, kucoin_client: &Arc<Client>, gate_state: Arc<GateState>, kucoin_state: Arc<KucoinState>) {

    let news_url = "ws://54.248.8.86:5050";
    let krypto_url = Url::parse(news_url).expect("Invalid WebSocket URL");
    //let (processed_tx, processed_rx) = broadcast::channel::<WsMessage>(500);
    let (processed_tx, processed_rx) = broadcast::channel::<String>(500);

    //let (processed_tx, _processed_rx) = broadcast::channel::<WsMessage>(500);
    let processed_tx_clone = processed_tx.clone();

    let keep_alive_gate = tokio::spawn(keep_alive_transaction(Arc::clone(&gate_client)));

    let keep_alive_kucoin = tokio::spawn(keep_alive_kucoin(Arc::clone(&kucoin_client)));

    let gate_clone = gate_client.clone();
    let kucoin_clone = kucoin_client.clone();
    let kucoin_state_clone = kucoin_state.clone();
    let gate_state_clone = gate_state.clone();


    let server_task = tokio::spawn(async move {
/*
        let kucoin_clone = kucoin_client.clone();
        let kucoin_state_clone = kucoin_state.clone();
        //let gate_clone = gate_client.clone();
        let gate_state_clone = gate_state.clone(); */


        loop {
            match connect_to_websocket(&krypto_url, processed_tx_clone.clone(), &kucoin_clone, &kucoin_state_clone, &gate_clone, &gate_state_clone).await {
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
    let _ = keep_alive_gate.await;
    let _ = keep_alive_kucoin.await;
    println!("All tasks completed.");
}



/*


    //let gate_client = Arc::new(Client::new());
    //let gate_state = Arc::new(GateState::new());
    //let kucoin_client = Arc::new(Client::new());
    //let kucoin_state = Arc::new(KucoinState::new());*/