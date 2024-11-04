use axum::{middleware::from_fn_with_state, Router};

use crate::{event::Sender, middleware::counter::counter};

pub(crate) async fn proxy_server(sender: Sender) {
    let app = Router::new().layer(from_fn_with_state(sender.clone(), counter));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
