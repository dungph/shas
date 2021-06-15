mod connection;

use seed::{prelude::*, *};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    Model {
        connection: connection::Model::init(&[], &mut orders.proxy(Msg::Connection)),
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

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
