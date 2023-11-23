use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use sha2::{Sha512, Digest};
use hmac::{Hmac, Mac, NewMac};
use serde_json::json;
use std::time::{ SystemTime, UNIX_EPOCH };
use tokio::time::sleep;
use std::time::Duration;
use std::sync::Arc;


type HmacSha512 = Hmac<Sha512>;

const BASE_URL: &str = "https://fx-api.gateio.ws";
const REQUEST_URL: &str = "/api/v4/futures/usdt/orders";  // Hardcoded settle as "usdt"
const METHOD: &str = "POST";
const ORDER_TYPE: &str = "limit";
const ORDER_SIDE: &str = "buy";
const CANCEL: &str = "ioc";


const API_KEY: &[u8] = b"f148b05a23ae3deb2c01b1fae2ad90bc";
const SECRET: &[u8] = b"79867bb850db0b5aac3776ef6752ff55f819462cfaf32ad5398749635c790884";


pub async fn keep_alive_transaction(gate_client: Arc<Client>) {
    loop {
        let currency_pair = "DOGE_USDT".to_string();
        let price: f64 = 0.068;
        let amount_as_integer: i64 = 380;
       
        let query_string = "";
        let body = json!({
            "contract": currency_pair,
            "type": "limit",
            "tif": "ioc",  // Immediate-Or-Cancel
            "side": "buy",           // Change this based on the real action
            "size": amount_as_integer.to_string(),
            "price": price.to_string()
        });

        let payload_hash = format!("{:x}", Sha512::digest(body.to_string().as_bytes()));
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        let signature_string = format!("{}\n{}\n{}\n{}\n{}", METHOD, REQUEST_URL, query_string, payload_hash, timestamp);   
    
        let mut mac = HmacSha512::new_varkey(SECRET).expect("HMAC can take key of any size");
        mac.update(signature_string.as_bytes());
        let result = mac.finalize();
        let signed_string = format!("{:x}", result.into_bytes());
    
        let mut headers = HeaderMap::new();
        headers.insert("KEY", HeaderValue::from_static(
            std::str::from_utf8(API_KEY).expect("Failed to convert API key to string")
        ));
    
        headers.insert("SIGN", HeaderValue::from_str(&signed_string).unwrap());
        headers.insert("Timestamp", HeaderValue::from_str(&timestamp.to_string()).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        

        let url = format!("{}{}", BASE_URL, REQUEST_URL);
        let res = gate_client
                        .post(&url)
                        .headers(headers)
                        .body(body.to_string())
                        .send()
                        .await
                        .unwrap();

            let _response_text = res.text().await.unwrap();

        sleep(Duration::from_secs(8)).await;
    }
}

pub async fn execute_gate_buy(gate_client: &Client, currency_pair: &str, price: f64, amount_as_integer: i64) -> f64 {

    let query_string = "";
    let body = format!(
        r#"{{"contract": "{}_USDT", "type": "{}", "side": "{}", "size": "{}", "price": "{}", "tif": "{}"}}"#,
        currency_pair, 
        ORDER_TYPE, 
        ORDER_SIDE, 
        amount_as_integer, 
        price, 
        CANCEL
    );

    let payload_hash = format!("{:x}", Sha512::digest(body.to_string().as_bytes()));
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    let signature_string = format!("{}\n{}\n{}\n{}\n{}", METHOD, REQUEST_URL, query_string, payload_hash, timestamp);   

    let mut mac = HmacSha512::new_varkey(SECRET).expect("HMAC can take key of any size");
    mac.update(signature_string.as_bytes());
    let result = mac.finalize();
    let signed_string = format!("{:x}", result.into_bytes());

    let mut headers = HeaderMap::new();
    headers.insert("KEY", HeaderValue::from_static(
        std::str::from_utf8(API_KEY).expect("Failed to convert API key to string")
    ));

    headers.insert("SIGN", HeaderValue::from_str(&signed_string).unwrap());
    headers.insert("Timestamp", HeaderValue::from_str(&timestamp.to_string()).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));


    let url = format!("{}{}", BASE_URL, REQUEST_URL);
    let res = gate_client
                    .post(&url)
                    .headers(headers)
                    .body(body.to_string())
                    .send()
                    .await
                    .unwrap();

    let response_text = res.text().await.unwrap();

    println!("Response from buy ordergate: {}", response_text);

    amount_as_integer as f64
}



pub async fn execute_gate_sell(gate_client: &Client, currency_pair: &String, bought_amount: f64) {

    let query_string = "";

    let body = json!({
        "contract": currency_pair,
        "type": "market",
        "side": "sell",
        "tif": "ioc",
        "price": 0,
        "size": bought_amount,
        "close": false,
        "reduce_only": true
    });

   
    let payload_hash = format!("{:x}", Sha512::digest(body.to_string().as_bytes()));
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    let signature_string = format!("{}\n{}\n{}\n{}\n{}", METHOD, REQUEST_URL, query_string, payload_hash, timestamp);   

    let mut mac = HmacSha512::new_varkey(SECRET).expect("HMAC can take key of any size");
    mac.update(signature_string.as_bytes());
    let result = mac.finalize();
    let signed_string = format!("{:x}", result.into_bytes());

    let mut headers = HeaderMap::new();
    headers.insert("KEY", HeaderValue::from_static(
        std::str::from_utf8(API_KEY).expect("Failed to convert API key to string")
    ));

    headers.insert("SIGN", HeaderValue::from_str(&signed_string).unwrap());
    headers.insert("Timestamp", HeaderValue::from_str(&timestamp.to_string()).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

   
    let url = format!("{}{}", BASE_URL, REQUEST_URL);
    let res = gate_client
                    .post(&url)
                    .headers(headers)
                    .body(body.to_string())
                    .send()
                    .await
                    .unwrap();

    let response_text = res.text().await.unwrap();
    println!("Response from sellGate: {}", response_text);
    

}