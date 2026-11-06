use thiserror::Error;

#[derive(Error, Debug)]
pub enum GmiError {
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Legal transition rejected: {0}")]
    TransitionRejected(String),
    #[error("Stability law violation: {0}")]
    StabilityViolation(String),
    #[error("Trajectory linkage failed: {0}")]
    TrajectoryLinkageFailed(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Adapter error: {0}")]
    AdapterError(String),
    #[error("Invalid numeric value: {0}")]
    InvalidNumeric(String),
    #[error("Generic error: {0}")]
    Generic(String),
}
