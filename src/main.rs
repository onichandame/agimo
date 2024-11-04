use counter::counter;
use event::Event;
use futures::{pin_mut, FutureExt};
use probe::probe;
use proxy_server::proxy_server;
use scaler_server::scaler_server;
use tokio::sync::broadcast;

mod counter;
mod event;
mod externalscaler;
mod middleware;
mod probe;
mod proxy_server;
mod scaler_api;
mod scaler_server;

#[tokio::main]
async fn main() {
    let (sender, _) = broadcast::channel::<Event>(100);
    let proxy = proxy_server(sender.clone()).fuse();
    let scaler = scaler_server(sender.clone()).fuse();
    let counter = counter(sender.clone()).fuse();
    let probe = probe(sender.clone()).fuse();
    pin_mut!(proxy, scaler, counter);
    tokio::select! {
        _ = proxy => {},
        _ = scaler => {},
        _ = counter => {},
        _ = probe => {},
    }
}
