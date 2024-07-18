use config::ServiceType;

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Queryer {
    #[cfg(feature = "prometheus")]
    prometheus_endpoint: String,
    client: reqwest::Client,
}

impl Queryer {
    #[cfg(feature = "prometheus")]
    pub fn new(prometheus_address: &str) -> Self {
        Self {
            prometheus_endpoint: format!("{}/api/v1/query", prometheus_address),
            client: reqwest::Client::new(),
        }
    }

    pub async fn query(&self, ty: &ServiceType, namespace: &str, name: &str) -> Result<u64> {
        let metric_name = match ty {
            ServiceType::Deployment => "kube_deployment_status_replicas_ready",
            ServiceType::Statefulset => "kube_statefulset_status_replicas_ready",
        };
        let app_type = match ty {
            ServiceType::Deployment => "deployment",
            ServiceType::Statefulset => "statefulset",
        };
        let res = self
            .client
            .get(self.prometheus_endpoint.as_str())
            .query(&[(
                "query",
                format!(
                    "{}{{namespace=\"{}\",{}=\"{}\"}}",
                    metric_name, namespace, app_type, name
                )
                .as_str(),
            )])
            .send()
            .await?;
        if res.status() != 200 {
            return Err(Error::HTTPError {
                code: res.status().as_u16(),
                msg: res.text().await?,
            });
        }
        let res = res.json::<serde_json::Value>().await?;
        let val = res
            .get("data")
            .and_then(|v| v.get("result"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("value"))
            .and_then(|v| v.get(1))
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or(Error::MetricInvalid)?;
        Ok(val)
    }
}

#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("Query fail: {code} {msg}")]
    HTTPError { code: u16, msg: String },
    #[error("Query fail: {0}")]
    QueryError(#[from] reqwest::Error),
    #[error("Metric invalid")]
    MetricInvalid,
}
