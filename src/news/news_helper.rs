use tokio::sync::RwLock;
use std::time::Instant;
use tokio::time::Duration;
use std::sync::Arc;



pub async fn extract(s: &str) -> Option<String> {
    println!("Received Message: {}", s);


    let extractors: Vec<fn(&str) -> Option<String>> = vec![
        binance_fut,
        upbit,
        bithumb,
        binance_listning,
        paribu,
        launches,
        coinbasesupport,
        binance_contract_swap,
        coinbaseassets,
        coinbase,
        binance_us
        
    ];
    
    for extractor in extractors.iter() {
        if let Some(result) = extractor(s) {
            return Some(result);
        }
    }
    
    None
}

pub fn binance_contract_swap(s: &str) -> Option<String> {
    if s.contains("Binance Will Support the ") && s.contains(" Contract Swap") || s.contains(" Token Swap") {
        if let Some(start) = s.find('(') {
            if let Some(end) = s.find(')') {
                let token = &s[(start + 1)..end];
                return Some(token.to_string());
            }
        }
    }
    None
}

pub fn binance_fut(s: &str) -> Option<String> {
    if let Some(start) = s.find("Binance Futures Will Launch USDⓈ-M ") {
        let start_idx = start + "Binance Futures Will Launch USDⓈ-M ".len();
        let end_idx = s[start_idx..].find(' ').unwrap_or(s.len());
        let token = &s[start_idx..start_idx + end_idx];
        return Some(token.to_string());
    }
    None
}


pub fn binance_listning(s: &str) -> Option<String> {
    if s.contains("Binance Will List") {

        if let Some(start) = s.find('(') {
            if let Some(end) = s.find(')') {
                let token = &s[(start + 1)..end];
                return Some(token.to_string());
            }
        }
    }
    None
}


pub fn launches(s: &str) -> Option<String> {
    if s.contains("Introducing") && s.contains("Binance Launchpad") || s.contains("Binance Launchpool") {
        return Some("BNB".to_string());

    }
    None
}


pub fn upbit(s: &str) -> Option<String> {
    if s.contains("[거래]") && (s.contains("KRW") || s.contains("KRW, BTC") || s.contains("BTC")) {

        if let Some(start) = s.find('(') {
            if let Some(end) = s.find(')') {
                let token = &s[(start + 1)..end];
                return Some(token.to_string());
            }
        }
    }
    None
}


pub fn bithumb(s: &str) -> Option<String> {
    if s.contains("[마켓 추가]") || s.contains("[투자유의]") {
        if let Some(start) = s.find('(') {
            if let Some(end) = s.find(')') {
                let token = &s[(start + 1)..end];
                return Some(token.to_string());
            }
        }
    }
    None
}

pub fn paribu(s: &str) -> Option<String> {
    if s.contains("işlemleri başladı") {

        if let Some(start) = s.find('(') {
            if let Some(end) = s.find(')') {
                let token = &s[(start + 1)..end];
                return Some(token.to_string());
            }
        }
    }
    None
}

pub fn coinbasesupport(s: &str) -> Option<String> {
    if s.contains("Coinbase will add support for") {
        if let Some(start) = s.find('(') {
            if let Some(end) = s.find(')') {
                let token = &s[(start + 1)..end];
                return Some(token.to_string());
            }
        }
    }
    None
}

pub fn coinbaseassets(s: &str) -> Option<String> {
    if s.contains("Asset added") && s.contains("roadmap today") {

        if let Some(start) = s.find('(') {
            if let Some(end) = s.find(')') {
                let token = &s[(start + 1)..end];
                return Some(token.to_string());
            }
        }
    }
    None
}

pub fn coinbase(s: &str) -> Option<String> {
    if s.contains("Coinbase exchange listing") {

        if let Some(start) = s.find('(') {
            if let Some(end) = s.find(')') {
                let token = &s[(start + 1)..end];
                return Some(token.to_string());
            }
        }
    }
    None
}


pub fn binance_us(s: &str) -> Option<String> {
    if s.contains("Binance.US Lists") {

        if let Some(start) = s.find('(') {
            if let Some(end) = s.find(')') {
                let token = &s[(start + 1)..end];
                return Some(token.to_string());
            }
        }
    }
    None
}


pub async fn cool_off_function(token: &str, last_processed: Arc<RwLock<(String, Instant)>>) -> bool {
    {
        let read_guard = last_processed.read().await;
        let (last_token, time) = &*read_guard;
        if token == last_token && time.elapsed() < Duration::from_secs(300) {  // 3 minutes cool-off
            return false;
        }
    }
    
    {
        let mut write_guard = last_processed.write().await;
        *write_guard = (token.to_string(), Instant::now());
    }
    
    true
}
