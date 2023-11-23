use std::time::SystemTime;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use std::error::Error;
use reqwest::Client;
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue};
use tokio::time::Instant;
use tokio::time::sleep;
use std::time::Duration;
extern crate base64;


const API_KEY: &str = "6559e9bc66315b0001cdaa12";
const API_SECRET: &str = "f74b8f7f-690e-4d2b-90d3-6af7429a8311";
const API_PASSPHRASE: &str = "Kolmder123!";
const API_VERSION: &str = "2";

const LEVERAGE: &str = "10";
const SIDE: &str = "buy";
const ORDER_TYPE: &str = "limit";
const TIME: &str = "IOC";

const METHOD: &str = "POST";
const ENDPOINT: &str = "/api/v1/orders";
const API_ENDPOINT: &str = "https://api-futures.kucoin.com/api/v1/orders";




fn common_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("KC-API-KEY", HeaderValue::from_static(API_KEY));
    headers.insert("KC-API-KEY-VERSION", HeaderValue::from_static(API_VERSION));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers
}


fn get_request_headers(sign: &str, timestamp_millis: &str, passphrase_base64: &str) -> HeaderMap {
    let mut headers = common_headers();  // Get a fresh copy each time
    headers.insert("KC-API-SIGN", HeaderValue::from_str(sign).unwrap());
    headers.insert("KC-API-TIMESTAMP", HeaderValue::from_str(timestamp_millis).unwrap());
    headers.insert("KC-API-PASSPHRASE", HeaderValue::from_str(passphrase_base64).unwrap());
    headers
}

pub fn create_signature(timestamp_millis: &str, method: &str, endpoint: &str, params: &str) -> String {
    let message = format!("{}{}{}{}", timestamp_millis, method, endpoint, params);
    let mut mac = Hmac::<Sha256>::new_varkey(API_SECRET.as_bytes()).unwrap();
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    base64::encode(&code_bytes)
}


pub async fn keep_alive_kucoin(client: Arc<Client>) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {

    loop {
        let timestamp_millis = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis()
            .to_string();
        let client_oid = timestamp_millis; // Using the timestamp as clientOid


        let body = serde_json::json!({
            "clientOid": client_oid,
            "side":"buy",
            "orderType":"limit",
            "symbol":"DOGEUSDTM",
            "size":"10",
            "price":"0.062",
            "timeInForce":"IOC",
            "leverage":"5"
        });

        let body_str = body.to_string();
        let sign = create_signature(&client_oid, "POST", "/api/v1/orders", &body_str);

    let mut mac = Hmac::<Sha256>::new_varkey(API_SECRET.as_bytes()).unwrap();
    mac.update(API_PASSPHRASE.as_bytes());
    let passphrase_hash = mac.finalize().into_bytes();
    let passphrase_base64 = base64::encode(&passphrase_hash);

    let mut headers = HeaderMap::new();
    headers.insert("KC-API-KEY", HeaderValue::from_static(API_KEY));
    headers.insert("KC-API-SIGN", HeaderValue::from_str(&sign)?);
    headers.insert("KC-API-TIMESTAMP", HeaderValue::from_str(&client_oid)?);
    headers.insert("KC-API-PASSPHRASE", HeaderValue::from_str(&passphrase_base64)?);
    headers.insert("KC-API-KEY-VERSION", HeaderValue::from_static(API_VERSION));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let body_str = body.to_string();
    let response = client.post(API_ENDPOINT)
        .headers(headers)
        .body(body_str)
        .send()
        .await?;

        let _response_text = response.text().await?;
        sleep(Duration::from_secs(7)).await;  
    }
}



pub async fn execute_kucoin_order(client: &Client, currency_pair: &str, adjusted_price: f64, size: i64) -> Result<i64, Box<dyn Error>> {
    println!("bought amount {}", size);
    println!("Price {}", adjusted_price);

    let timestamp_millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis()
        .to_string();

    let client_oid = timestamp_millis;





        let body = format!(
        r#"{{"clientOid":"{}","side":"{}","symbol":"{}USDTM","type":"{}","timeInForce":"{}","leverage":{},"size":{},"price":{}}}"#,
        client_oid, SIDE, currency_pair, ORDER_TYPE, TIME, LEVERAGE, size, adjusted_price
    );


/*

    let body = format!(
        r#"{{"clientOid":"{}","side":"{}","symbol":"{}USDTM","type":"{}","timeInForce":"{}","leverage":{},"size":{},"price":0.8200}}"#,
        client_oid, SIDE, currency_pair, ORDER_TYPE, TIME, LEVERAGE, size
    ); */

    
    let sign = create_signature(&client_oid, "POST", "/api/v1/orders", &body);

    let mut mac = Hmac::<Sha256>::new_varkey(API_SECRET.as_bytes()).unwrap();
    mac.update(API_PASSPHRASE.as_bytes());
    let passphrase_hash = mac.finalize().into_bytes();
    let passphrase_base64 = base64::encode(&passphrase_hash);

    let headers = get_request_headers(&sign, &client_oid, &passphrase_base64); // Clone and update headers

    let start_time = Instant::now();
    let response = client.post(API_ENDPOINT)
        .headers(headers)
        .body(body)
        .send()
        .await?;

    let duration = start_time.elapsed();
    println!("Request duration: {:?}", duration);


    let response_text = response.text().await?;
    println!("Kucoin buy order response: {}", response_text);

    Ok(size)
}


pub async fn execute_sell_order(client: &Client, currency_pair: &str, amount: f64) -> Result<(), Box<dyn Error>> {

    println!("qty sell '{}':", amount);
    println!("symbol '{}':", currency_pair);


    let timestamp_millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        .to_string();

        let client_oid = timestamp_millis; // Using the timestamp as clientOid


        let body = format!(
            r#"{{"clientOid":"{}","side":"sell","symbol":"{}","type":"market","timeInForce":"IOC","leverage":10,"size":{},"reduceOnly":"true"}}"#,
            client_oid, currency_pair, amount
        );
  
    // Create the signature
    let sign = create_signature(&client_oid, METHOD, ENDPOINT, &body);
    
    // Hash the passphrase
    let mut mac = Hmac::<Sha256>::new_varkey(API_SECRET.as_bytes()).unwrap();
    mac.update(API_PASSPHRASE.as_bytes());
    let passphrase_hash = mac.finalize().into_bytes();
    let passphrase_base64 = base64::encode(&passphrase_hash);
    
    // Get the headers
    let headers = get_request_headers(&sign, &client_oid, &passphrase_base64);
    
    // Execute the POST request
    let response = client.post(API_ENDPOINT)
        .headers(headers)
        .body(body)
        .send()
        .await?;

    let response_text = response.text().await?;
    println!("Kucoin sell response text: {}", response_text); // Print the response text

    Ok(())
}