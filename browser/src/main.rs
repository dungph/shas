mod connection;

use seed::{prelude::*, *};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    let static_key: [u8; 32] = if let Ok(x) = LocalStorage::get("static_dh") {
        x
    } else {
        LocalStorage::insert("static_dh", &rand::random::<[u8; 32]>()).unwrap();
        LocalStorage::get("static_dh").unwrap()
    };
    Model {
        connection: connection::Model::init(static_key, &mut orders.proxy(Msg::Connection)),
    }
}

pub struct Model {
    connection: connection::Model,
}

pub enum Msg {
    Connection(connection::Msg),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Connection(msg) => {
            model
                .connection
                .update(msg, &mut orders.proxy(Msg::Connection));
        }
    }
    if let Some(msg) = model.connection.recv() {
        log!(msg)
    }
}

fn view(model: &Model) -> Node<Msg> {
    model.connection.view().map_msg(Msg::Connection)
}

fn main() {
    App::start("app", init, update, view);
}
