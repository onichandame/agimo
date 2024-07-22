use clap::{Args, Parser};

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about)]
pub struct CliArgs {
    #[cfg(feature = "prometheus")]
    #[command(flatten)]
    pub prometheus: PrometheusConfig,
    /// Path to config file
    ///
    /// example: ./config.toml
    #[arg(long, env = "AGIMO_CONF")]
    pub conf: String,
    /// Port for service
    #[arg(long, default_value_t = 8080)]
    pub port: u16,
}

#[cfg(feature = "prometheus")]
#[derive(Clone, Debug, Args)]
pub struct PrometheusConfig {
    /// Prometheus server address
    ///
    /// example: http://127.0.0.1:9090
    #[arg(long = "prometheus-address", env = "AGIMO_PROMETHEUS_ADDRESS")]
    pub address: String,
    #[arg(long = "prometheus-exporter-port", default_value_t = 9090)]
    pub exporter_port: u16,
}
