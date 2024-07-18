use async_trait::async_trait;
use config::{Config, Service};
use pingora::{
    prelude::fast_timeout::{fast_sleep, fast_timeout},
    protocols::Digest,
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
};
use prometheus::IntCounterVec;
#[cfg(feature = "prometheus")]
use prometheus::IntGaugeVec;

use crate::queryer::{self, Queryer};

type Result<T> = std::result::Result<T, Error>;

pub struct Interceptor {
    /// total number of pending requests labeled by host
    #[cfg(feature = "prometheus")]
    pending_gauge: IntGaugeVec,
    /// total number of active requests labeled by host
    #[cfg(feature = "prometheus")]
    active_gauge: IntGaugeVec,
    #[cfg(feature = "prometheus")]
    total_counter: IntCounterVec,
    queryer: Queryer,
    services: Config,
}

impl Interceptor {
    #[cfg(feature = "prometheus")]
    pub fn new(prometheus_endpoint: &str, services: Config) -> Result<Self> {
        use prometheus::{opts, register_int_counter_vec, register_int_gauge_vec};
        let pending_gauge = register_int_gauge_vec!(
            opts!("agimo_pending_requests", "pending requests"),
            &["host"]
        )
        .unwrap();
        let active_gauge =
            register_int_gauge_vec!(opts!("agimo_active_requests", "active requests"), &["host"])
                .unwrap();
        let total_counter =
            register_int_counter_vec!(opts!("agimo_requests_total", "total requests"), &["host"])
                .unwrap();
        let queryer = {
            #[cfg(feature = "prometheus")]
            Queryer::new(prometheus_endpoint)
        };
        Ok(Self {
            pending_gauge,
            active_gauge,
            total_counter,
            queryer,
            services,
        })
    }
}

#[async_trait]
impl ProxyHttp for Interceptor {
    type CTX = Context;

    fn new_ctx(&self) -> Self::CTX {
        Context::default()
    }

    /// hold until upstream ready
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        let host = session
            .get_header("host")
            .ok_or(Error::HostInvalid)?
            .to_str()
            .map_err(|_| Error::HostInvalid)?;
        let Some(service) = self.services.services.iter().find(|s| s.host == host) else {
            return Err(Error::ServiceNotFound {
                host: host.to_owned(),
            }
            .into());
        };
        ctx.service = Some(service.to_owned());
        self.pending_gauge.with_label_values(&[host]).inc();
        self.total_counter.with_label_values(&[host]).inc();
        let timeout = &service.timeout.unwrap_or(self.services.timeout);
        let Ok(_) = fast_timeout(*timeout.to_owned(), async {
            'LOOP: loop {
                let ready = self
                    .queryer
                    .query(&service.ty, &service.namespace, &service.name)
                    .await
                    .map_err(Into::<Error>::into)?;
                if ready > 0 {
                    break 'LOOP;
                };
                fast_sleep(std::time::Duration::from_secs(1)).await;
            }
            Ok::<(), Error>(())
        })
        .await
        else {
            return Err(Error::ServiceNotReady {
                host: host.to_owned(),
                reason: "timeout".to_owned(),
            }
            .into());
        };
        Ok(false)
    }

    /// hold the request until the current load is under the service capacity, or time out. Then discharge the request to the service.
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let Some(service) = &ctx.service else {
            return Err(Error::ServiceNotFound {
                host: "".to_owned(),
            }
            .into());
        };
        let peer = Box::new(HttpPeer::new(
            format!("{}:{}", service.service_name, service.service_port),
            false,
            "".to_owned(),
        ));
        Ok(peer)
    }

    async fn connected_to_upstream(
        &self,
        _session: &mut Session,
        _reused: bool,
        _peer: &HttpPeer,
        _fd: std::os::unix::io::RawFd,
        _digest: Option<&Digest>,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()> {
        let Some(service) = &ctx.service else {
            return Err(Error::ServiceNotFound {
                host: "".to_owned(),
            }
            .into());
        };
        ctx.active = true;
        self.active_gauge.with_label_values(&[&service.host]).inc();
        Ok(())
    }

    async fn logging(
        &self,
        _session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let Some(service) = &ctx.service else {
            return;
        };
        self.pending_gauge.with_label_values(&[&service.host]).dec();
        if ctx.active {
            self.active_gauge.with_label_values(&[&service.host]).dec();
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("Service not found: {host:?}")]
    ServiceNotFound { host: String },
    #[error("Host header invalid")]
    HostInvalid,
    #[error("Service {host:?} not ready: {reason:?}")]
    ServiceNotReady { host: String, reason: String },
    #[error("Read services error:  {0}")]
    ReadServicesError(#[from] std::io::Error),
    #[error("Query error:  {0}")]
    QueryError(#[from] queryer::Error),
}

impl From<Error> for pingora::BError {
    fn from(err: Error) -> Self {
        let status_code: u16 = match &err {
            Error::ServiceNotFound { host: _ } => 404,
            Error::HostInvalid => 400,
            Error::ServiceNotReady { host: _, reason: _ } => 503,
            _other => 500,
        };
        pingora::Error::explain(pingora::HTTPStatus(status_code), err.to_string())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Context {
    service: Option<Service>,
    active: bool,
}
