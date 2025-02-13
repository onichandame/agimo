use futures::{pin_mut, FutureExt};

mod event_bus;
mod externalscaler;
mod scaler_api;

mod cache;
mod constant;
mod k8s_api;
mod proxy_server;
mod replica_monitor;
mod scaler_server;

#[tokio::main]
async fn main() {
    let event_bus = event_bus::EventBus::new();
    let cache = cache::cache(event_bus.clone()).fuse();
    let replica_monitor = replica_monitor::replica_monitor(event_bus.clone()).fuse();
    let k8s_api = k8s_api::k8s_api(event_bus.clone()).fuse();
    let proxy = proxy_server::proxy_server(event_bus.clone()).fuse();

    let scaler = scaler_server::scaler_server(event_bus.clone()).fuse();
    pin_mut!(cache, replica_monitor, proxy, scaler);
    tokio::select! {
        _ = cache => {},
        _ = replica_monitor => {},
        _ = k8s_api => {},
        _ = proxy => {},
        _ = scaler => {},
    }
}
