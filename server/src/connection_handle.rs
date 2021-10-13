use async_std::{prelude::StreamExt, sync::Mutex};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::{collections::HashMap, io, sync::Arc};
use tide::{Request, Result};
use tide_websockets::{Message, WebSocketConnection as Connection};
use utils::{decode_cbor, encode_cbor};

type Pool = HashMap<Vec<u8>, Arc<Mutex<dyn ObjSender>>>;
static POOL: Lazy<Mutex<Pool>> = Lazy::new(|| Mutex::new(HashMap::new()));

async fn insert_sender(key: &[u8], sender: impl ObjSender + 'static) -> Arc<Mutex<dyn ObjSender>> {
    let sender = Arc::new(Mutex::new(sender));
    let p = POOL.lock();
    p.await.insert(key.to_vec(), sender.clone());
    sender
}

pub async fn run(_req: Request<()>, stream: Connection) -> Result<()> {
    let mut read_stream = stream.clone().filter_map(|message| match message {
        Ok(Message::Binary(b)) => Some(b),
        _ => None,
    });

    let b = read_stream
        .next()
        .await
        .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, ""))?;

    let mut payload = vec![0u8; 1024];
    let e = rand::random::<[u8; 32]>();
    let (_, responder) = noise_ix::responder(e, *crate::vars::PKEY, &[])
        .read_message(&b, &mut payload)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, ""))?;

    let remote_key = responder.remote_key();

    let mut msg = [0u8; 96];
    let (len, transport) = responder
        .write_message(&[], &mut msg)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, ""))?;
    stream.send_bytes(msg[..len].to_vec()).await?;

    let (mut noise_read, noise_write) = transport.split();
    let sender = insert_sender(&remote_key, (stream.clone(), noise_write)).await;

    while let Some(bytes) = read_stream.next().await {
        let mut payload = vec![0u8; bytes.len() - 16];

        noise_read
            .read_message(&bytes, &mut payload)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, ""))?;

        let payload: Value =
            decode_cbor(&payload).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, ""))?;
        //echo back
        sender.lock().await.send(payload).await?;
    }
    Ok(())
}

#[async_trait::async_trait]
pub(crate) trait ObjSender: Send + Sync {
    async fn send(&mut self, obj: Value) -> Result<()>;
}

#[async_trait::async_trait]
impl ObjSender for (Connection, noise_ix::NoiseWrite) {
    async fn send(&mut self, obj: Value) -> Result<()> {
        let mut buf = [0u8; 1024];
        let written = encode_cbor(&obj, &mut buf).unwrap();

        let mut message = Vec::new();
        message.resize(written + 16, 0u8);
        self.1.write_message(&buf[..written], &mut message).unwrap();

        self.0.send_bytes(message).await?;
        Ok(())
    }
}
