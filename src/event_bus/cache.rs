use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub(crate) enum CacheRequest {
    /// refresh self
    RefreshSelf,
    /// get replica map from cache
    GetReplicaMap,
    /// remove dead replica
    RemoveReplica(String),
    /// increment a counter
    Inc(String),
    /// decrement a counter
    Dec(String),
    /// get so
    GetSO(String),
    /// downplay so
    DownplaySO(String),
}

#[derive(Debug, Clone)]
pub(crate) enum CacheResponse {
    /// refresh self
    RefreshSelf,
    /// replica map
    GetReplicaMap(ReplicaMap),
    /// remove dead replica
    RemoveReplica,
    /// increment a counter
    Inc,
    /// decrement a counter
    Dec,
    /// get so
    GetSO(Option<SO>),
    /// downplay so
    DownplaySO,
}

pub(crate) type ReplicaMap = std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SO {
    metadata: SOMetadata,
    spec: SOSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SOMetadata {
    name: String,
    namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SOSpec {
    scale_target_ref: SOScaleTargetRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SOScaleTargetRef {
    name: String,
    namespace: String,
    kind: String,
    api_version: String,
}
