mod gate;
mod kucoin;
//mod bybit;
mod state;
mod exec;
mod news;
mod server;


use crate::exec::gate_exec::run_gate;
use crate::exec::kucoin_exec::run_kucoin;
use crate::exec::server_exec::run_server;
use crate::state::kucoin_state::KucoinState;
use crate::state::gate_state::GateState;


use reqwest::Client;
use std::sync::Arc;


#[tokio::main]
async fn main() {

    let kucoin_lesser = Arc::new(Client::new());
    let kucoin_client = Arc::new(Client::new());
    let kucoin_state = Arc::new(KucoinState::new());

    let gate_lesser = Arc::new(Client::new());
    let gate_client = Arc::new(Client::new());
    let gate_state = Arc::new(GateState::new());


    let gate_state_clone = Arc::clone(&gate_state);
    tokio::spawn(async move {
        run_gate(&gate_lesser, gate_state_clone).await;
        // Since run_gate returns (), there is no error handling needed here
    });

    let kucoin_state_clone = Arc::clone(&kucoin_state);
    tokio::spawn(async move {
        if let Err(e) = run_kucoin(&kucoin_lesser, kucoin_state_clone).await {
            eprintln!("Error running kucoin: {:?}", e);
        }
    });

    let new_gate_state = Arc::clone(&gate_state);
    let new_kucoin_state = Arc::clone(&kucoin_state);
    tokio::spawn(async move {
        run_server(&gate_client, &kucoin_client, new_gate_state, new_kucoin_state).await;
        // Since run_server returns (), there is no error handling needed here
    });


    futures::future::pending::<()>().await;
}


/*

    let gate_test = Arc::clone(&gate_state);
    let test_task = tokio::spawn(async move {
       tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
       // Uncomment if you need to use these lines
       //bybit_state.print_bybit_contract("DOGEUSDT").await;
       //tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

       // Assuming "HIFI" is the value you want to test with
       let test_value = "MBL".to_string(); // Use the exact key as stored in the state
       // You would need to ensure `client` and `bybit_state` are available in this scope
       gate_calc(&test, &gate_test, test_value).await;
    });


    let kucoin_test = Arc::clone(&kucoin_state);
    let test_task2 = tokio::spawn(async move {
       tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
       // Uncomment if you need to use these lines
       //bybit_state.print_bybit_contract("DOGEUSDT").await;
       //tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

       // Assuming "HIFI" is the value you want to test with
       let test_value = "MBL".to_string(); // Use the exact key as stored in the state
       // You would need to ensure `client` and `bybit_state` are available in this scope
       kucoin_calc(&test2, &kucoin_test, test_value).await;
    });



   test_task.await.expect("Test task failed");
   test_task2.await.expect("Test task failed");*/