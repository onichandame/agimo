use crate::{constant::HEARTBEAT_INTERVAL, event_bus};

/// Periodically checks the state of the replicas. Remove dead replicas and refresh self.
///
/// - Remove dead replicas when they are not seen for more than 2 heartbeat intervals
/// - Refresh self every 30 seconds
pub(crate) async fn replica_monitor(event_bus: event_bus::EventBus) -> Result<()> {
    loop {
        event_bus
            .request(event_bus::Request::CacheRequest(
                event_bus::CacheRequest::RefreshSelf,
            ))
            .await?;
        if let event_bus::Response::CacheResponse(event_bus::CacheResponse::GetReplicaMap(repl_map)) =
            event_bus
                .request(event_bus::Request::CacheRequest(
                    event_bus::CacheRequest::GetReplicaMap,
                ))
                .await?
        {
            for (host_id, last_seen) in repl_map {
                if last_seen
                    < chrono::Utc::now() - chrono::Duration::seconds(HEARTBEAT_INTERVAL * 2)
                {
                    event_bus
                        .request(event_bus::Request::CacheRequest(
                            event_bus::CacheRequest::RemoveReplica(host_id),
                        ))
                        .await?;
                }
            }
        } else {
            return Err(event_bus::Error::WrongResponse.into());
        }

        // Sleep for 30 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(HEARTBEAT_INTERVAL as u64)).await;
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Event error: {0}")]
    EventError(#[from] event_bus::Error),
}
