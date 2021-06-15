use std::{collections::HashMap, convert::TryInto, io, sync::Arc};

//use crate::entity::MsgSender;
//use anyhow::anyhow;
use async_std::{
    channel::{unbounded, Sender},
    prelude::StreamExt,
    sync::Mutex,
};
use futures::future::{select, Either};
use once_cell::sync::Lazy;
use payload::Payload;
use tide::{Request, Result};
//use tide_websockets::Message::Close;
use sha2::{self, Digest};
use tide_websockets::{Message, WebSocketConnection as Connection};
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

use crate::peer_handle::{handle_payload, insert_sender};

const NOISE_XX: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";

pub async fn run(_req: Request<()>, mut stream: Connection) -> Result<()> {
    let mut sha256 = sha2::Sha256::default();
    sha256.update(&*crate::vars::SECRET);
    let key = sha256.finalize();

    let mut noise = snow::Builder::new(NOISE_XX.parse().unwrap())
        .local_private_key(&key)
        .build_responder()
        .unwrap();

    println!("\n handshake started");
    // handshake
    while !noise.is_handshake_finished() {
        if noise.is_my_turn() {
            let mut msg = [0u8; 96];
            let len = noise.write_message(&[], &mut msg).unwrap();
            stream.send_bytes(msg[..len].to_vec()).await?;
        } else {
            if let Some(Ok(Message::Binary(message))) = stream.next().await {
                let mut payload = vec![0u8; 1024];
                noise.read_message(&message, &mut payload)?;
            } else {
                Err(io::Error::new(io::ErrorKind::Other, ""))?
            }
        }
    }

    println!("\n handshake finished");
    let mut noise = noise.into_transport_mode()?;

    let remote_key = noise.get_remote_static().unwrap();

    let (sender, receiver) = unbounded();

    // for testing
    sender.send(Payload::ConnectionAccepted).await?;
    insert_sender(&remote_key, sender).await;

    let send_stream = stream.clone();
    let mut out_fut = receiver.recv();
    let mut in_fut = stream.next();

    loop {
        match select(in_fut, out_fut).await {
            Either::Left((msg, old_out_fut)) => {
                if let Some(Ok(Message::Binary(bytes))) = msg {
                    let mut payload = Vec::new();
                    payload.resize(bytes.len() - 16, 0u8);

                    noise.read_message(&bytes, &mut payload)?;

                    let payload: Payload = serde_cbor::from_slice(&payload)?;

                    let peer = noise.get_remote_static().unwrap();

                    handle_payload(peer, payload).await?;
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, ""))?
                }

                in_fut = stream.next();
                out_fut = old_out_fut;
            }
            Either::Right((payload, old_in_fut)) => {
                let payload = serde_cbor::to_vec(&payload?).unwrap();
                let mut message = Vec::new();
                message.resize(payload.len() + 16, 0u8);

                noise.write_message(&payload, &mut message)?;

                send_stream.send_bytes(message).await?;

                in_fut = old_in_fut;
                out_fut = receiver.recv();
            }
        }
    }
}
