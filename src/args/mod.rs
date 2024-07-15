use clap::{Args, Parser};

#[derive(Clone, Debug, Parser)]
pub struct CliArgs {
    #[cfg(feature = "prometheus")]
    #[command(flatten)]
    pub prometheus: PrometheusConfig,
    /// Path to config file
    ///
    /// example: ./config.toml
    #[arg(long)]
    pub conf: String,
    /// Port for service
    #[arg(long, default_value_t = 8080)]
    pub port: u16,
    #[arg(long, default_value_t = false)]
    pub upgrade: bool,
}

#[cfg(feature = "prometheus")]
#[derive(Clone, Debug, Args)]
pub struct PrometheusConfig {
    /// Prometheus server address
    ///
    /// example: http://127.0.0.1:9090
    #[arg(long = "prometheus-address")]
    pub address: String,
    #[arg(long = "prometheus-exporter-port", default_value_t = 9090)]
    pub exporter_port: u16,
}
