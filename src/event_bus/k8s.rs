#[derive(Debug, Clone)]
pub(crate) enum K8sRequest {
    GetReadyReplicas(String),
}

#[derive(Debug, Clone)]
pub(crate) enum K8sResponse {
    GetReadyReplicas(u64),
}
