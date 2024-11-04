use std::future::Future;

pub(super) trait Counter: Clone + Send + Sync {
    type Error: Send + Sync;
    /// increment request count for the host
    fn inc(&self, host: &str) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// decrement request count for the host
    fn dec(&self, host: &str) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// get active request count for the host
    fn get_active(&self, host: &str) -> impl Future<Output = Result<u64, Self::Error>> + Send;
}
