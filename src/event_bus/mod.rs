mod cache;
mod k8s;

use std::pin::Pin;

use futures::{Future, StreamExt};

pub(crate) use cache::*;
pub(crate) use k8s::*;

pub(crate) enum Message {
    Request(Request),
    Event(Event),
}

#[derive(Debug, Clone)]
pub(crate) enum Request {
    CacheRequest(CacheRequest),
    K8sRequest(K8sRequest),
}

#[derive(Debug, Clone)]
pub(crate) enum Response {
    CacheResponse(CacheResponse),
    K8sResponse(K8sResponse),
}

#[derive(Debug, Clone)]
pub(crate) enum Event {}

#[derive(Debug, Clone)]
enum InternalEvent {
    Request(Request, tokio::sync::mpsc::Sender<Response>),
    Response(Response),
}

#[derive(Debug, Clone)]
pub(crate) struct EventBus {
    sender: tokio::sync::broadcast::Sender<InternalEvent>,
}

impl EventBus {
    pub(crate) fn new() -> Self {
        Self {
            sender: tokio::sync::broadcast::channel(100).0,
        }
    }

    /// Send a request and wait for a response
    pub(crate) async fn request(&self, request: Request) -> Result<Response> {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        self.sender.send(InternalEvent::Request(request, sender))?;
        match receiver.recv().await {
            Some(response) => Ok(response),
            None => Err(Error::NoResponse),
        }
    }

    pub(crate) fn subscribe(
        &self,
    ) -> impl futures::Stream<
        Item = (
            Request,
            Box<dyn FnOnce(Response) -> Pin<Box<dyn Future<Output = Result<()>>>>>,
        ),
    > {
        let subscriber = self.sender.subscribe();
        Box::pin(
            tokio_stream::wrappers::BroadcastStream::new(subscriber).filter_map(
                |event| async move {
                    match event {
                        Ok(InternalEvent::Request(request, responder)) => Some((
                            request,
                            Box::new(move |res| {
                                Box::pin(async move { Ok(responder.send(res).await?) })
                                    as Pin<Box<dyn Future<Output = Result<()>>>>
                            })
                                as Box<
                                    dyn FnOnce(
                                        Response,
                                    )
                                        -> Pin<Box<dyn Future<Output = Result<()>>>>,
                                >,
                        )),
                        _other => None,
                    }
                },
            ),
        )
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error(
        "Cannot send internal event. This normally indicates either a bug or a broken runtime"
    )]
    SendError(#[from] tokio::sync::broadcast::error::SendError<InternalEvent>),
    #[error("Internal error: Cannot respond to event. This normally indicates either a bug or a broken runtime")]
    RespondError(#[from] tokio::sync::mpsc::error::SendError<Response>),
    #[error("No response")]
    NoResponse,
    #[error("Timeout error: {0}")]
    TimeoutError(#[from] tokio::time::error::Elapsed),
    #[error("Wrong response: The response was not expected. This normally indicates either a bug")]
    WrongResponse,
}
