mod connection_handle;
mod database;
mod peer_handle;
mod vars;

use tide_websockets::WebSocket;

pub fn app() -> anyhow::Result<tide::Server<()>> {
    let mut app = tide::new();

    app.at("/").get(tide::Redirect::new("/index.html"));
    app.at("/").serve_dir("../browser/dist/")?;
    app.at("/ws").get(WebSocket::new(connection_handle::run));
    Ok(app)
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    tide::log::start();
    database::migrate().await?;
    app()?
        .listen(format!("0.0.0.0:{}", *vars::WEB_PORT))
        .await?;
    Ok(())
}
