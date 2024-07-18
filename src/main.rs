use agimo_core::Interceptor;
use args::CliArgs;
use clap::Parser;
use config::Config;
use pingora::{proxy::http_proxy_service_with_name, server::Server, services::listening::Service};

mod args;

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let mut server = Server::new(None)?;
    let services = parse_services(&args.conf)?;
    let mut interceptor = http_proxy_service_with_name(
        &server.configuration,
        Interceptor::new(&args.prometheus.address, services)?,
        "Interceptor",
    );
    interceptor.add_tcp(format!("0.0.0.0:{}", &args.port).as_str());
    server.add_service(interceptor);
    #[cfg(feature = "prometheus")]
    {
        let mut prom = Service::prometheus_http_service();
        prom.add_tcp(format!("0.0.0.0:{}", &args.prometheus.exporter_port).as_str());
        server.add_service(prom);
    }
    server.run_forever();
}

fn parse_services(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let services: Config = toml::from_str(&content)?;
    Ok(services)
}
