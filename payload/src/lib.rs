use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_cbor::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "t", rename_all = "camelCase")]
pub enum Payload {
    ConnectionAccepted,
    ConnectionDenied,

    AskAdminAccept {
        peer: Vec<u8>,
    },
    AdminAccept {
        peer: Vec<u8>,
    },

    #[cfg(feature = "login")]
    Login {
        admin_pwd: String,
    },

    AskData,
    SyncData(Data),
    SyncRequest(Data),

    #[cfg(feature = "relay")]
    Relay(Relay),
}

#[cfg(feature = "relay")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relay {
    pub src: Vec<u8>,
    pub dest: Vec<u8>,
    pub dat: Vec<u8>,
}

pub type Data = BTreeMap<Vec<u8>, BTreeMap<Vec<u8>, Value>>;
