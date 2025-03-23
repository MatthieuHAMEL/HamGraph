use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::rolling::RollingFileAppender;

use crate::infraglobals;

static mut LOG_GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;

pub fn init_logger(to_file: bool, log_level: &str) {
  let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new(log_level));

  if to_file {
    let file_appender: RollingFileAppender = tracing_appender::rolling::daily(&infraglobals::get_logger_path(), "hamgraph.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    unsafe { LOG_GUARD = Some(guard); } // There must be better ... :) 
    fmt().with_env_filter(filter).with_writer(non_blocking).with_ansi(false).init();
  } 
  else { // Stdout
    fmt().with_env_filter(filter).init();
  }
}
