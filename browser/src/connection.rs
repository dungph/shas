use std::collections::VecDeque;

use noise_ix::{Initiator2, Transport};
use seed::{prelude::*, *};
use serde_json::Value;
use utils::{decode_cbor, encode_cbor};

pub struct Model {
    private_key: [u8; 32],
    text: String,
    handshake: Option<Initiator2>,
    transport: Option<Transport>,
    ws: WebSocket,
    payload: VecDeque<Value>,
}

#[derive(Clone)]
pub enum Msg {
    Text(String),
    Send,
    Recv(Vec<u8>),
    Disconnected,
    Connected,
}

impl Model {
    pub fn init(key: [u8; 32], orders: &mut impl Orders<Msg>) -> Self {
        Model {
            private_key: key,
            text: String::new(),
            handshake: None,
            transport: None,
            ws: ws_open(orders),
            payload: VecDeque::new(),
        }
    }
    pub fn send(&mut self, payload: Value) {
        if let Some(ref mut state) = self.transport {
            let mut buf = [0u8; 1024];
            let written = encode_cbor(&payload, &mut buf).unwrap();

            let mut message = vec![0u8; written + 16];
            let len = state.write_message(&buf[..written], &mut message).unwrap();
            self.ws.send_bytes(&message[..len]).unwrap();
        };
    }
    pub fn recv(&mut self) -> Option<Value> {
        self.payload.pop_front()
    }
    pub fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::Text(s) => self.text = s,
            Msg::Send => {
                let json = serde_json::json!({
                    "text": self.text
                });
                self.send(json);
            }
            Msg::Recv(message) => {
                if let Some(state) = self.handshake.take() {
                    let mut payload = vec![0u8; 512];
                    let (_, trans) = state.read_message(&message[..], &mut payload).unwrap();

                    self.transport = Some(trans);
                } else {
                    let mut payload = vec![0u8; message.len() - 16];
                    if let Some(ref mut state) = self.transport {
                        let len = state.read_message(&message, &mut payload).unwrap();
                        let payload = decode_cbor(&payload[..len]).unwrap();
                        self.payload.push_back(payload)
                    }
                }
            }
            Msg::Disconnected => {
                log!("disconnected");
                self.transport = None;
            }
            Msg::Connected => {
                log!("connected");
                let e = rand::random::<[u8; 32]>();
                let init1 = noise_ix::initiator(e, self.private_key, &[]);

                let mut message = vec![0u8; 64];
                let (len, init2) = init1.write_message(&[], &mut message).unwrap();
                self.ws.send_bytes(&message[..len]).unwrap();
                self.handshake = Some(init2);
            }
        }
    }

    pub fn view(&self) -> Node<Msg> {
        div![
            C!["section"],
            div![
                C!["field has-addons"],
                label!["",],
                div![
                    C!["control"],
                    input![
                        C!["input"],
                        attrs! {
                            At::Placeholder => "Text",
                            At::Value => self.text,
                            At::Type => "text",
                        },
                        input_ev(Ev::Input, Msg::Text),
                        keyboard_ev(Ev::KeyDown, |keyboard_event| {
                            IF!(keyboard_event.key_code() == 13 => Msg::Send)
                        }),
                    ]
                ],
                div![
                    C!["control"],
                    button![
                        C!["button is-primary"],
                        "Connect",
                        input_ev(Ev::Click, |_| Msg::Send),
                    ]
                ]
            ],
        ]
    }
}

pub fn ws_open(orders: &mut impl Orders<Msg>) -> WebSocket {
    let msg_send1 = orders.msg_sender();
    let msg_send2 = orders.msg_sender();
    let msg_send3 = orders.msg_sender();

    WebSocket::builder(format!("ws://{}/ws", &hostname()), orders)
        .on_message(move |m| {
            spawn_local(async move {
                msg_send1(Some(Msg::Recv(m.bytes().await.unwrap())));
            });
        })
        .on_close(move |_| {
            msg_send2(Some(Msg::Disconnected));
        })
        .on_open(move || {
            msg_send3(Some(Msg::Connected));
        })
        .build_and_open()
        .unwrap()
}

pub fn hostname() -> String {
    let loc = web_sys::window().unwrap().location();
    let host = loc.hostname();
    let port = loc.port();

    if port.is_ok() {
        format!("{}:{}", host.unwrap(), port.unwrap())
    } else {
        host.unwrap()
    }
}
