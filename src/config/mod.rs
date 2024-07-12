use clap::{Args, Parser};

#[derive(Clone, Debug, Parser)]
pub struct Config {
    #[cfg(feature = "prometheus")]
    #[command(flatten)]
    pub prometheus: PrometheusConfig,
    /// Path to services config
    ///
    /// example: ./services.yaml
    #[arg(long)]
    pub services: String,
    /// Port for service
    #[arg(long, default_value_t = 8080)]
    pub port: u16,
}

#[cfg(feature = "prometheus")]
#[derive(Clone, Debug, Args)]
pub struct PrometheusConfig {
    /// Prometheus endpoint
    ///
    /// example: http://127.0.0.1:9090
    #[arg(long = "prometheus-endpoint")]
    pub endpoint: String,
}
