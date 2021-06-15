use std::{collections::HashMap, io};

use async_std::{channel::Sender, sync::Mutex};
use once_cell::sync::Lazy;
use payload::Payload;

static POOL: Lazy<Mutex<HashMap<Vec<u8>, Sender<Payload>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn insert_sender(key: &[u8], sender: Sender<Payload>) {
    POOL.lock().await.insert(key.to_vec(), sender);
}

pub async fn handle_payload(peer: &[u8], payload: Payload) -> io::Result<()> {
    match payload {
        Payload::ConnectionAccepted => todo!(),
        Payload::ConnectionDenied => todo!(),
        Payload::AskAdminAccept { peer } => todo!(),
        Payload::AdminAccept { peer } => todo!(),
        Payload::Login { admin_pwd } => todo!(),
        Payload::AskData => todo!(),
        Payload::SyncData(_) => todo!(),
        Payload::SyncRequest(_) => todo!(),
        Payload::Relay(_) => todo!(),
    };
    Ok(())
}
