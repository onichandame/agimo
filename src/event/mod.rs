use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Clone)]
pub(crate) enum Event {
    /// Emitted when an incoming request is received
    InitRequest {
        host: String,
    },
    /// Emitted when an incoming request is closed
    CloseRequest {
        host: String,
    },
    /// Emit when the active request count for a host is updated
    UpdateActive {
        host: String,
        active: u64,
    },

    // below are request-response events
    /// Get the active request count for a host from the counter
    GetActive {
        host: String,
        sender: mpsc::Sender<u64>,
    },
    GetReplicas {
        host: String,
        sender: mpsc::Sender<u64>,
    },
}

pub(crate) type Sender = broadcast::Sender<Event>;
