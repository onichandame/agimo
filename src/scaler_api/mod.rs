use std::pin::Pin;

use futures::{stream::once, Stream, StreamExt};
use tokio::sync::{broadcast::Sender, mpsc};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

use crate::{
    externalscaler::{
        external_scaler_server::ExternalScaler, GetMetricSpecResponse, GetMetricsRequest,
        GetMetricsResponse, IsActiveResponse, MetricSpec, MetricValue, ScaledObjectRef,
    },
};

const METRIC_NAME: &str = "requests_count";

pub(crate) struct ScalerApi {
    event_sender: Sender<Event>,
}

impl ScalerApi {
    pub(crate) fn new(event_sender: Sender<Event>) -> Self {
        Self { event_sender }
    }

    fn get_host(so_ref: &ScaledObjectRef) -> Result<String, Status> {
        so_ref
            .scaler_metadata
            .get("host")
            .map(|s| s.to_string())
            .ok_or(Status::invalid_argument("host not provided"))
    }

    fn get_threshold(so_ref: &ScaledObjectRef) -> Result<i64, Status> {
        so_ref
            .scaler_metadata
            .get("threshold")
            .and_then(|s| s.parse().ok())
            .ok_or(Status::invalid_argument("threshold not provided"))
    }

    async fn get_active_request_count(&self, host: &str) -> Result<u64, Status> {
        let (sender, mut recv) = mpsc::channel(1);
        self.event_sender
            .send(Event::GetActive {
                host: host.to_string(),
                sender,
            })
            .map_err(|_| Status::internal("event send error"))?;
        recv.recv().await.ok_or(Status::internal("recv error"))
    }
}

#[tonic::async_trait]
impl ExternalScaler for ScalerApi {
    async fn is_active(
        &self,
        request: Request<ScaledObjectRef>,
    ) -> Result<Response<IsActiveResponse>, Status> {
        let host = Self::get_host(request.get_ref())?;
        Ok(Response::new(IsActiveResponse {
            result: self.get_active_request_count(&host).await? > 0,
        }))
    }

    type StreamIsActiveStream =
        Pin<Box<dyn Stream<Item = Result<IsActiveResponse, Status>> + Send>>;

    async fn stream_is_active(
        &self,
        request: Request<ScaledObjectRef>,
    ) -> Result<Response<Self::StreamIsActiveStream>, Status> {
        let host = Self::get_host(request.get_ref())?;
        let is_active = self.get_active_request_count(&host).await?;
        let stream = once(async move { is_active > 0 })
            .then(|result| async move { Ok(IsActiveResponse { result }) })
            .chain(
                BroadcastStream::new(self.event_sender.subscribe()).filter_map(move |res| {
                    let host = host.clone();
                    async move {
                        if let Ok(event) = res {
                            match event {
                                Event::UpdateActive {
                                    host: updated_host,
                                    active,
                                } => {
                                    if updated_host == host {
                                        return Some(Ok(IsActiveResponse { result: active > 0 }));
                                    }
                                }
                                _other => {}
                            }
                        } else {
                            return Some(Err(Status::internal("event system error")));
                        }
                        None
                    }
                }),
            );
        Ok(Response::new(stream.boxed()))
    }

    async fn get_metric_spec(
        &self,
        request: Request<ScaledObjectRef>,
    ) -> Result<Response<GetMetricSpecResponse>, Status> {
        let threshold = Self::get_threshold(request.get_ref())?;
        Ok(Response::new(GetMetricSpecResponse {
            metric_specs: vec![MetricSpec {
                metric_name: METRIC_NAME.to_string(),
                target_size: threshold,
            }],
        }))
    }

    async fn get_metrics(
        &self,
        request: Request<GetMetricsRequest>,
    ) -> Result<Response<GetMetricsResponse>, Status> {
        let host = Self::get_host(
            request
                .get_ref()
                .scaled_object_ref
                .as_ref()
                .ok_or(Status::invalid_argument("ScaledObjectRef not provided"))?,
        )?;
        Ok(Response::new(GetMetricsResponse {
            metric_values: vec![MetricValue {
                metric_name: METRIC_NAME.to_string(),
                metric_value: self
                    .get_active_request_count(&host)
                    .await?
                    .try_into()
                    .map_err(|_| Status::out_of_range("metric value out of range"))?,
            }],
        }))
    }
}
