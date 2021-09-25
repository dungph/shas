use std::{collections::HashMap, io};

use async_std::{channel::Sender, sync::Mutex};
use once_cell::sync::Lazy;
use serde_json::Value;

static POOL: Lazy<Mutex<HashMap<Vec<u8>, Sender<Value>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn insert_sender(key: &[u8], sender: Sender<Value>) {
    POOL.lock().await.insert(key.to_vec(), sender);
}

pub async fn handle_payload(_peer: &[u8], _payload: Value) -> io::Result<()> {
    Ok(())
}
