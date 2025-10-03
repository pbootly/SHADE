use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Clone)]
pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        // Initialize tracing subscriber globally
        fmt().with_env_filter(EnvFilter::from_default_env()).init();

        info!("Logger initialized—SHADE server shall speak.");
        Logger
    }

    pub fn info(&self, msg: &str) {
        info!("{}", msg);
    }

    pub fn error(&self, msg: &str) {
        error!("{}", msg);
    }
}
