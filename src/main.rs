mod gate;
mod kucoin;
//mod bybit;
mod state;
mod exec;
mod news;
mod server;


use crate::exec::gate_exec::run_gate;
//use crate::exec::bybit_exec::run_bybit;
use crate::exec::kucoin_exec::run_kucoin;
use crate::exec::server_exec::run_server;



#[tokio::main]
async fn main() {
    let (_, kucoin_result, _) = tokio::join!(
        run_gate(),
        run_kucoin(),
        run_server(), // Assuming run_server is also an async function
        // You can continue to add more calls here for other exchanges or tasks
    );

    // Handle the result of run_kucoin
    if let Err(e) = kucoin_result {
        eprintln!("Error running kucoin: {:?}", e);
    } 

    // Add any additional cleanup or finalization code here if necessary
}

/*
#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());
    let shared_state = Arc::new(SharedState::new());

    match get_futures_contracts(client.clone()).await {
        Ok(usdt_contracts) => {
            shared_state.update_contracts(usdt_contracts);

            let ws_url = Url::parse("wss://fx-ws.gateio.ws/v4/ws/usdt").expect("Failed to parse WebSocket URL");
            let shared_state_for_websocket = shared_state.clone();

            // Spawn a new task for the WebSocket connection
            tokio::spawn(async move {
                loop {
                    match connect_gate_websocket(&ws_url, shared_state_for_websocket.clone()).await {
                        Ok(()) => {
                            let now = Utc::now();
                            eprintln!("{} - WebSocket connection closed cleanly, will attempt to reconnect...", now.to_rfc3339());
                        },
                        Err(e) => {
                            let now = Utc::now();
                            eprintln!("{} - Error with WebSocket connection: {:?}", now.to_rfc3339(), e);
                        }
                    }
            
                    // Reconnection backoff
                    eprintln!("{} - Attempting to reconnect to WebSocket...", Utc::now().to_rfc3339());
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            });

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await; // Sleep for 1 hour
                let now = Utc::now();
                println!("time {}...", now);
                shared_state.print_specific_contract("DOGE_USDT").await;           
           
             }
        },
        Err(e) => {
            eprintln!("Failed to get futures contracts: {:?}", e);
        },
    }



    
} */