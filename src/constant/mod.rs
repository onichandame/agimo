/// The seconds of heartbeat interval
///
/// **!!! This value should not exceed the max bound of i64**
pub(crate) const HEARTBEAT_INTERVAL: i64 = 30;

/// The seconds of time window
pub(crate) const TIME_WINDOW: i64 = 3600 * 24 * 30;
