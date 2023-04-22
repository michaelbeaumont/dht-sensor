#[derive(Debug)]
pub enum DhtError {
    PinError,
    ChecksumMismatch,
    Timeout,
}
