use axum::{
    extract::{Host, Request, State},
    http::StatusCode,
    middleware::Next,
};

use crate::event::{Event, Sender};

pub(crate) async fn counter(
    State(sender): State<Sender>,
    Host(host): Host,
    req: Request,
    next: Next,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    sender
        .send(Event::InitRequest { host: host.clone() })
        .unwrap();
    let res = next.run(req).await;
    sender
        .send(Event::CloseRequest { host: host.clone() })
        .unwrap();
    Ok(res)
}
