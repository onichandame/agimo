use std::collections::HashMap;

use base64::{prelude::BASE64_STANDARD, Engine};
use futures::StreamExt;
use redis::AsyncCommands;

use crate::{
    constant::{HEARTBEAT_INTERVAL, TIME_WINDOW},
    event_bus::{self, CacheRequest, CacheResponse, EventBus, Request, Response},
};

const REPLICA_MAP_KEY: &str = "replicas";
const SO_KEY_PREFIX: &str = "so";

pub(crate) async fn cache(bus: EventBus) -> Result<()> {
    let replica_id = uuid::Uuid::new_v4().to_string();
    let client = redis::Client::open(std::env::var("REDIS_URL").expect("REDIS_URL must be set"))?;
    let mut con = client.get_multiplexed_async_connection().await?;
    while let Some((Request::CacheRequest(request), respond)) = bus.subscribe().next().await {
        match request {
            CacheRequest::RefreshSelf => {
                let _: () = redis::pipe()
                    .hset(REPLICA_MAP_KEY, &replica_id, chrono::Utc::now().timestamp())
                    .expire(REPLICA_MAP_KEY, HEARTBEAT_INTERVAL * 2)
                    .query_async(&mut con)
                    .await?;
                respond(Response::CacheResponse(CacheResponse::RefreshSelf)).await?;
            }
            CacheRequest::GetReplicaMap => {
                let res: HashMap<String, i64> = con.hgetall(REPLICA_MAP_KEY).await?;
                let mut repl_map = HashMap::new();
                for (host_id, timestamp) in res {
                    if let Some(last_seen) = chrono::DateTime::from_timestamp(timestamp, 0) {
                        repl_map.insert(host_id, last_seen);
                    } else {
                        // This should never happen
                        let _: () = con.hdel("replicas", host_id).await?;
                    }
                }

                respond(Response::CacheResponse(CacheResponse::GetReplicaMap(
                    repl_map,
                )))
                .await?;
            }
            CacheRequest::RemoveReplica(host_id) => {
                let _: () = redis::pipe()
                    .hdel(REPLICA_MAP_KEY, host_id)
                    .expire(REPLICA_MAP_KEY, HEARTBEAT_INTERVAL * 2)
                    .query_async(&mut con)
                    .await?;
                respond(Response::CacheResponse(CacheResponse::RemoveReplica)).await?;
            }
            CacheRequest::Inc(host) => {
                let counter_key = get_counter_key(&host);
                let active_counter_key = get_active_counter_key(&host);
                let _: () = redis::pipe()
                    .zadd(&counter_key, 1, chrono::Utc::now().timestamp())
                    .zrembyscore(
                        &counter_key,
                        0,
                        chrono::Utc::now().timestamp() - TIME_WINDOW,
                    )
                    .expire(&counter_key, TIME_WINDOW)
                    .incr(&active_counter_key, 1)
                    .expire(&active_counter_key, TIME_WINDOW)
                    .query_async(&mut con)
                    .await?;
                respond(Response::CacheResponse(CacheResponse::Inc)).await?;
            }
            CacheRequest::Dec(host) => {
                let active_counter_key = get_active_counter_key(&host);
                let _: () = redis::pipe()
                    .decr(&active_counter_key, 1)
                    .query_async(&mut con)
                    .await?;
                respond(Response::CacheResponse(CacheResponse::Dec)).await?;
            }
            CacheRequest::GetSO(host) => {
                let so_key = get_so_key(&host);
                let res: String = con.get(so_key).await?;
                let so = serde_json::from_str(&res).ok();
                respond(Response::CacheResponse(CacheResponse::GetSO(so))).await?;
            }
            CacheRequest::DownplaySO(host) => {
                if so_map.contains_key(&host) {
                    so_map.insert(host.clone(), so_map[&host] - 1);
                    if so_map[&host] == 0 {
                        so_map.remove(&host);
                    }
                }
                respond(Response::CacheResponse(CacheResponse::DownplaySO)).await?;
            }
        }
    }
    Ok(())
}

fn get_counter_key(host: &str) -> String {
    format!("{{counter:{}}}", BASE64_STANDARD.encode(host))
}

fn get_active_counter_key(host: &str) -> String {
    format!("{}:active", get_counter_key(host))
}

fn get_so_key(host: &str) -> String {
    format!("{}:so", host)
}

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("Event error: {0}")]
    EventError(#[from] event_bus::Error),
    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
}
