use clap::Parser;
use agimo::{Config, Interceptor};
use pingora::{proxy::http_proxy_service_with_name, server::Server, services::listening::Service};

fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    let mut server = Server::new(None)?;
    let mut interceptor = http_proxy_service_with_name(
        &server.configuration,
        Interceptor::new(&config.prometheus.endpoint, &config.services)?,
        "Interceptor",
    );
    interceptor.add_tcp(format!("0.0.0.0:{}", &config.port).as_str());
    server.add_service(interceptor);
    #[cfg(feature = "prometheus")]
    {
        let mut prom = Service::prometheus_http_service();
        prom.add_tcp("0.0.0.0:9090");
        server.add_service(prom);
    }
    server.run_forever();
}
