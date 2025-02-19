pub mod file;

use std::sync::OnceLock;

use chrono::Local;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan; // for fmt::Write

use tracing_subscriber::fmt::format::Writer;
// use tracing_subscriber::fmt::{format, time::FormatTime};

// pub const APP_CODE: &str = "123123123123123";
// pub const APP_CODE: &str = "66a7577a2b2f3b0130375e6f";
pub const APP_CODE: OnceLock<String> = OnceLock::new();
static SN_CODE: once_cell::sync::Lazy<std::sync::RwLock<Option<String>>> =
    once_cell::sync::Lazy::new(|| std::sync::RwLock::new(None));

// 设置 SN 码
pub fn set_sn_code(sn: &str) {
    let mut sn_lock = SN_CODE.write().unwrap();
    *sn_lock = Some(sn.to_string());
}

// 获取 SN 码
pub fn get_sn_code() -> String {
    let sn_lock = SN_CODE.read().unwrap();
    sn_lock.clone().unwrap_or("sn".to_string())
}

static LOG_LEVEL: once_cell::sync::Lazy<std::sync::RwLock<Option<String>>> =
    once_cell::sync::Lazy::new(|| std::sync::RwLock::new(None));

pub fn set_log_level(level: &str) {
    let mut log_level_lock = LOG_LEVEL.write().unwrap();
    *log_level_lock = Some(level.to_string());
}

pub fn get_log_level() -> String {
    let log_level_lock = LOG_LEVEL.read().unwrap();
    log_level_lock
        .clone()
        .unwrap_or(file::DEFAULT_LOG_LEVEL.to_string())
}

pub fn init_test_log() {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .init();
}

pub fn init_log() {
    // 初始化日志输出，带有自定义时间格式和事件格式
    tracing_subscriber::fmt()
        .pretty()
        // .with_timer(CustomTime) // 自定义时间格式
        .with_target(true) // 显示日志目标
        .with_level(true) // 显示日志级别
        .event_format(CustomEventFormat) // 使用自定义的事件格式化器 // 通过闭包传递自定义的事件格式化函数
        // .fmt_fields(format::DefaultFields::new()) // 使用默认字段格式化器
        .init();
}

// struct CustomTime;

// impl FormatTime for CustomTime {
//     fn format_time(&self, w: &mut format::Writer<'_>) -> std::fmt::Result {
//         let now = chrono::Local::now(); // 获取当前本地时间
//         write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S")) // 自定义时间格式
//     }
// }

// 自定义事件格式实现
struct CustomEventFormat;

impl<S, N> FormatEvent<S, N> for CustomEventFormat
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();

        // 时间
        write!(writer, "{} ", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))?;

        // appcode
        write!(writer, "{} ", APP_CODE.get().ok_or(std::fmt::Error)?)?;

        // 日志级别
        write!(writer, "{} ", meta.level())?;

        // 操作系统信息
        write!(writer, "{} ", crate::system_info::get_os_info())?;

        // 日志目标
        write!(writer, "{} ", meta.target())?;

        // ip地址
        write!(writer, "127.0.0.1 ")?;

        // 设备sn
        write!(writer, "{} ", get_sn_code())?;

        // 事件字段
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

#[cfg(test)]
mod tests {

    // 初始化日志配置，供测试使用

    use tracing::info;

    use crate::init_log;

    #[test]
    fn test_logging_format() {
        // 初始化日志配置
        init_log();

        // 触发日志输出
        info!("访问xxx出错.............");
        info!("访问xxx出错.............");
    }
}
