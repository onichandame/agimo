use std::collections::HashMap;

use futures::StreamExt;
use kube::Client;

use crate::event_bus::{CacheRequest, EventBus, K8sRequest, K8sResponse, Request, Response};

pub(crate) async fn k8s_api(event_bus: EventBus) -> Result<()> {
    let client = Client::try_default().await?;
    let conn_count = HashMap::new();
    let host_replicas = HashMap::new();
    while let Some((request, respond)) = event_bus.subscribe().next().await {
        match request {
            Request::K8sRequest(k8s_request) => match k8s_request {
                K8sRequest::GetReadyReplicas(host) => {
                    respond(Response::K8sResponse(K8sResponse::GetReadyReplicas(
                        *host_replicas.get(&host).unwrap_or(&0),
                    )))
                    .await?;
                }
            },
            Request::CacheRequest(cache_request) => match cache_request {
                CacheRequest::Inc(host) => {
                    if !conn_count.contains_key(&host) {
                        tokio::spawn(async move {});
                    }
                    *conn_count.entry(host.clone()).or_insert(0u64) += 1;
                }
                CacheRequest::Dec(host) => {
                    if let Some(count) = conn_count.get_mut(&host) {
                        *count = (*count).saturating_sub(1).max(0);
                    }
                }
                _other => {}
            },
            _other => {}
        }
    }
    Ok(())
}

async fn listen_to_workload() {}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("K8s client error: {0}")]
    K8sClientError(#[from] kube::Error),
    #[error("Event error: {0}")]
    EventError(#[from] crate::event_bus::Error),
}

enum InternalEvent {}
