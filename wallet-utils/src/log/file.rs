const DEFAULT_LOG_SIZE: u64 = 7 * 1024 * 1024;
pub const DEFAULT_LOG_LEVEL: &str = "error";

pub static LOG_GUARD: once_cell::sync::Lazy<
    once_cell::sync::OnceCell<tracing_appender::non_blocking::WorkerGuard>,
> = once_cell::sync::Lazy::new(once_cell::sync::OnceCell::new);

pub fn init_log(_path: &str, level: Option<&str>) -> Result<(), crate::Error> {
    let level = level.unwrap_or(DEFAULT_LOG_LEVEL);
    super::set_log_level(level);
    LOG_GUARD.get_or_init(|| _init_log(level, _path));

    Ok(())
}

fn _init_log(level: &str, path: &str) -> WorkerGuard {
    use tracing_subscriber::{
        fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
        Registry,
    };
    let file_appender = tracing_appender::rolling::never(path, format!("{level}.txt"));
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let formatting_layer = fmt::layer()
        // .pretty()
        // .with_writer(std::io::stderr)
        // .with_writer(file_appender)
        .with_writer(non_blocking)
        .with_ansi(false)
        .event_format(CustomEventFormat)
        .fmt_fields(format::DefaultFields::new());
    Registry::default()
        .with(env_filter)
        // ErrorLayer 可以让 color-eyre 获取到 span 的信息
        .with(tracing_error::ErrorLayer::default())
        .with(FileCheckLayer {
            log_file_path: format!("{path}/{level}.log"),
            size_limit: DEFAULT_LOG_SIZE,
        })
        .with(fmt::layer())
        .with(formatting_layer)
        .init();
    tracing::info!("[init log] Init log success");
    guard
}

struct FileCheckLayer {
    log_file_path: String,
    size_limit: u64,
}
use tracing::{Event, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt::format, layer::Context, Layer};

use crate::log::CustomEventFormat;

impl<S> Layer<S> for FileCheckLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {
        // Check the file size each time the log is written
        if let Ok(metadata) = std::fs::metadata(&self.log_file_path) {
            tracing::debug!("Check File Size: {}", metadata.len());
            if metadata.len() > self.size_limit {
                let _ = std::fs::write(&self.log_file_path, b""); // Clear file
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::warn;

    #[test]
    fn test_logging_format() {
        // 初始化日志配置
        init_log("./", None).unwrap();

        // 触发日志输出
        warn!("访问xxx出错.............");
        warn!("访问xxx出错.............");
    }
}
