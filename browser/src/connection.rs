use std::collections::VecDeque;

use payload::Payload;
use seed::{prelude::*, *};
use snow::{HandshakeState, TransportState};

const NOISE_XX: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";

pub struct Model {
    private_key: Vec<u8>,
    remote_passwd: String,
    handshake: Option<HandshakeState>,
    transport: Option<TransportState>,
    ws: WebSocket,
    payload: VecDeque<Payload>,
}

#[derive(Clone)]
pub enum Msg {
    Passwd(String),
    Login,
    Recv(Vec<u8>),
    Disconnected,
    Connected,
}

impl Model {
    pub fn init(key: &[u8], orders: &mut impl Orders<Msg>) -> Self {
        Model {
            private_key: key.to_vec(),
            remote_passwd: String::new(),
            handshake: None,
            transport: None,
            ws: ws_open(orders),
            payload: VecDeque::new(),
        }
    }
    pub fn send(&mut self, payload: Payload) {
        if let Some(ref mut state) = self.transport {
            let payload = serde_cbor::to_vec(&payload).unwrap();
            let mut message = vec![0u8; 65535];
            let len = state.write_message(&payload, &mut message).unwrap();
            self.ws.send_bytes(&message[..len]).unwrap();
        };
    }
    pub fn recv(&mut self) -> Option<Payload> {
        self.payload.pop_front()
    }
    pub fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::Passwd(s) => self.remote_passwd = s,
            Msg::Login => {
                let payload = Payload::Login {
                    admin_pwd: self.remote_passwd.clone(),
                };

                let payload = serde_cbor::to_vec(&payload).unwrap();
                if let Some(ref mut state) = self.transport {
                    let mut message = vec![0u8; 65535];
                    let len = state.write_message(&payload, &mut message).unwrap();
                    self.ws.send_bytes(&message[..len]).unwrap();
                }
            }
            Msg::Recv(message) => {
                log!("recv", message);

                if let Some(mut state) = self.handshake.take() {
                    let mut payload = vec![0u8; 512];
                    state.read_message(&message[..], &mut payload).unwrap();

                    let message = &mut payload;
                    let len = state.write_message(&[], message).unwrap();
                    self.ws.send_bytes(&message[..len]).unwrap();

                    self.transport = Some(state.into_transport_mode().unwrap());
                    log!("transport mode");
                } else {
                    let mut payload = vec![0u8; 65535];
                    if let Some(ref mut state) = self.transport {
                        let len = state.read_message(&message, &mut payload).unwrap();
                        let payload = serde_cbor::from_slice(&payload[..len]).unwrap();
                        log!(payload);
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
                let mut state = snow::Builder::new(NOISE_XX.parse().unwrap())
                    .local_private_key(&self.private_key)
                    .build_initiator()
                    .unwrap();

                let mut message = vec![0u8; 48];
                let len = state.write_message(&[], &mut message).unwrap();
                self.ws.send_bytes(&message[..len]).unwrap();
                self.handshake = Some(state);
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
                            At::Placeholder => "Password",
                            At::Value => self.remote_passwd,
                            At::Type => "password",
                        },
                        input_ev(Ev::Input, |s| Msg::Passwd(s)),
                        keyboard_ev(Ev::KeyDown, |keyboard_event| {
                            IF!(keyboard_event.key_code() == 13 => Msg::Login)
                        }),
                    ]
                ],
                div![
                    C!["control"],
                    button![
                        C!["button is-primary"],
                        "Connect",
                        input_ev(Ev::Click, |_| Msg::Login),
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
        .on_close(move |e| {
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
        format!("{}", host.unwrap())
    }
}
