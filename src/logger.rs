use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub fn init_logger(to_file: bool, log_level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    if to_file {
        let file_appender: RollingFileAppender = tracing_appender::rolling::daily("logs", "hamgraph.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        // TODO: _guard should be used to flush logs before exit
        fmt()
            .with_env_filter(filter)
            .with_writer(non_blocking)
            .init();
    } else {
        fmt()
            .with_env_filter(filter)
            .init();
    }
}
