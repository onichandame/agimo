use axum::{
    extract::{Host, Request, State},
    http::StatusCode,
    middleware::{from_fn_with_state, Next},
    response::{IntoResponse, Response},
    Router,
};

use crate::event_bus::{self, EventBus};

pub(crate) async fn proxy_server(event_bus: EventBus) {
    let app = Router::new()
        .layer(from_fn_with_state(event_bus.clone(), counter))
        .layer(from_fn_with_state(event_bus.clone(), interceptor));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub(crate) async fn counter(
    State(event_bus): State<EventBus>,
    Host(host): Host,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse> {
    event_bus
        .request(event_bus::Request::CacheRequest(event_bus::CacheRequest::Inc(
            host.clone(),
        )))
        .await?;
    let res = next.run(req).await;
    event_bus
        .request(event_bus::Request::CacheRequest(event_bus::CacheRequest::Dec(
            host.clone(),
        )))
        .await?;
    Ok(res)
}

pub(crate) async fn interceptor(
    State(event_bus): State<EventBus>,
    Host(host): Host,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse> {
    let so = event_bus
        .request(event_bus::Request::CacheRequest(
            event_bus::CacheRequest::GetReplicaMap,
        ))
        .await?;
    if let event_bus::Response::CacheResponse(event_bus::CacheResponse::GetSO(Some(so))) = so {
    } else {
        return Err(Error::ServiceNotFound { host: host.clone() });
    }
    let res = next.run(req).await;
    Ok(res)
}

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Event error: {0}")]
    EventError(#[from] event_bus::Error),
    #[error("Service not found: {host:?}")]
    ServiceNotFound { host: String },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
    }
}
