use duration_string::DurationString;
use std::time::Duration;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Services {
    pub(crate) services: Vec<Service>,
    /// How long to wait for upstream service to be ready
    ///
    /// default: 30s
    #[serde(default = "default_timeout", )]
    pub(crate) timeout: DurationString,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Service {
    /// The hostname of the service
    ///
    /// Any requests to this host will be proxied to the service
    pub(crate) host: String,
    #[serde(rename = "type")]
    pub(crate) ty: ServiceType,
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) service_name: String,
    pub(crate) service_port: u16,
    pub(crate) timeout: Option<DurationString>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) enum ServiceType {
    #[serde(rename = "deployment", alias = "deploy")]
    Deployment,
    #[serde(rename = "statefulset", alias = "sts")]
    Statefulset,
}

fn default_timeout() -> DurationString {
    Duration::from_secs(30).into()
}
