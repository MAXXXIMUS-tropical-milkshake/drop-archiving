use std::time::{Duration, Instant};
use tokio::time::error::Elapsed;
use tracing::{debug, error, info, span, subscriber, warn, Level, Span};
use tracing_subscriber::{filter::LevelFilter, fmt, layer::SubscriberExt, Layer, Registry};
use std::sync::Once;
use once_cell::sync::Lazy;

pub static LOGGER: Lazy<Logger> = Lazy::new(|| Logger::new("debug"));
pub struct Logger {
    _span: Span,
}

impl Logger {
    pub fn new(level: &str) -> Self {
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            let level = match level.to_lowercase().as_str() {
                "error" => LevelFilter::ERROR,
                "warn" => LevelFilter::WARN,
                "info" => LevelFilter::INFO,
                "debug" => LevelFilter::DEBUG,
                _ => LevelFilter::INFO,
            };

            let subscriber = Registry::default().with(
                fmt::layer()
                    .with_writer(std::io::stdout)
                    .with_filter(level),
            );

            tracing::subscriber::set_global_default(subscriber);
        });

        Self {
            _span: span!(Level::INFO, "application"),
        }

    }

    pub fn debug(&self, message: &str) {
        debug!(message);
    }

    pub fn info(&self, message: &str) {
        info!(message);
    }

    pub fn warn(&self, message: &str) {
        warn!(message);
    }
    
    pub fn error(&self, message: &str) {
        error!(message);
    }

    pub fn trace(&self, begin: Instant, sql: &str, rows: i64, err: Option<&str>) {
        let elapsed = begin.elapsed();
        let mut event = info!(sql, rows, elapsed = ?elapsed);
        if let Some(error) = err {
            event = error!(error);
        }
        event;
    }
}

