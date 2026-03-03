use thiserror::Error;

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Daemon error: {0}")]
    Daemon(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Monitor error: {0}")]
    Monitor(String),
}
