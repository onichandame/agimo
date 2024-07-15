use duration_string::DurationString;
use std::time::Duration;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub services: Vec<Service>,
    /// How long to wait for upstream service to be ready
    ///
    /// default: 30s
    #[serde(default = "default_timeout", )]
    pub timeout: DurationString,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Service {
    /// The hostname of the service
    ///
    /// Any requests to this host will be proxied to the service
    pub host: String,
    #[serde(rename = "type")]
    pub ty: ServiceType,
    pub namespace: String,
    pub name: String,
    pub service_name: String,
    pub service_port: u16,
    pub timeout: Option<DurationString>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum ServiceType {
    #[serde(rename = "deployment", alias = "deploy")]
    Deployment,
    #[serde(rename = "statefulset", alias = "sts")]
    Statefulset,
}

fn default_timeout() -> DurationString {
    Duration::from_secs(30).into()
}
