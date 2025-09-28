use std::path::Path;
use tracing::{debug, error, info, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 初始化日志系统
pub fn init_logging(
    log_level: &str,
    log_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 设置环境过滤器
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // 创建控制台输出层
    let console_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .compact();

    // 创建文件输出层（如果指定了日志文件）
    let file_layer = if let Some(log_file_path) = log_file {
        // 确保日志目录存在
        if let Some(parent) = Path::new(log_file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 创建滚动日志文件写入器
        let file_appender = rolling::daily(log_file_path, "matching_engine.log");
        let (non_blocking_appender, _guard) = non_blocking(file_appender);

        Some(
            fmt::layer()
                .with_writer(non_blocking_appender)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false)
                .json()
                .boxed(),
        )
    } else {
        None
    };

    // 初始化订阅者
    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer);

    if let Some(file_layer) = file_layer {
        registry.with(file_layer).init();
    } else {
        registry.init();
    }

    info!("Logging system initialized with level: {}", log_level);
    if let Some(log_file) = log_file {
        info!("Log file: {}", log_file);
    }

    Ok(())
}

// LoggingConfig 定义在 config.rs 中

/// 高级日志初始化
pub fn init_advanced_logging(
    config: crate::config::LoggingConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let mut layers = Vec::new();

    // 控制台输出层
    if config.console {
        let console_layer = if config.json_format {
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false)
                .json()
                .boxed()
        } else {
            fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(true)
                .compact()
                .boxed()
        };
        layers.push(console_layer);
    }

    // 文件输出层
    if let Some(log_file_path) = &config.file {
        // 确保日志目录存在
        if let Some(parent) = Path::new(log_file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 创建滚动日志文件写入器
        let file_appender = rolling::daily(log_file_path, "matching_engine.log");
        let (non_blocking_appender, _guard) = non_blocking(file_appender);

        let file_layer = fmt::layer()
            .with_writer(non_blocking_appender)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(false)
            .json()
            .boxed();

        layers.push(file_layer);
    }

    // 初始化订阅者
    let registry = tracing_subscriber::registry().with(env_filter);

    match layers.len() {
        0 => registry.init(),
        1 => registry.with(layers.into_iter().next().unwrap()).init(),
        _ => {
            let mut registry = registry;
            for layer in layers {
                registry = registry.with(layer);
            }
            registry.init();
        }
    }

    info!("Advanced logging system initialized");
    info!("Log level: {}", config.level);
    info!("Console output: {}", config.console);
    info!("JSON format: {}", config.json_format);
    if let Some(log_file) = &config.file {
        info!("Log file: {}", log_file);
    }

    Ok(())
}

/// 性能日志宏
#[macro_export]
macro_rules! perf_log {
    ($operation:expr, $duration:expr) => {
        tracing::info!(
            operation = $operation,
            duration_ms = $duration.as_millis(),
            "Performance: {} took {}ms",
            $operation,
            $duration.as_millis()
        );
    };
}

/// 交易日志宏
#[macro_export]
macro_rules! trade_log {
    ($trade:expr) => {
        tracing::info!(
            trade_id = %$trade.id,
            symbol = %$trade.symbol.to_string(),
            quantity = $trade.quantity,
            price = $trade.price,
            buyer_id = %$trade.buyer_id,
            seller_id = %$trade.seller_id,
            "Trade executed: {} {} at {} for {}",
            $trade.quantity,
            $trade.symbol.to_string(),
            $trade.price,
            $trade.id
        );
    };
}

/// 订单日志宏
#[macro_export]
macro_rules! order_log {
    ($order:expr, $action:expr) => {
        tracing::info!(
            order_id = %$order.id,
            symbol = %$order.symbol.to_string(),
            side = ?$order.side,
            order_type = ?$order.order_type,
            quantity = $order.quantity,
            price = $order.price,
            status = ?$order.status,
            user_id = %$order.user_id,
            "Order {}: {} {} {} {} at {}",
            $action,
            $order.side,
            $order.quantity,
            $order.symbol.to_string(),
            $order.order_type,
            $order.price.unwrap_or(0.0)
        );
    };
}

/// 错误日志宏
#[macro_export]
macro_rules! error_log {
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = %$error,
            context = $context,
            "Error in {}: {}",
            $context,
            $error
        );
    };
}

/// 统计日志宏
#[macro_export]
macro_rules! stats_log {
    ($stats:expr) => {
        tracing::info!(
            total_orders = $stats.total_orders,
            total_trades = $stats.total_trades,
            total_volume = $stats.total_volume,
            active_orders = $stats.active_orders,
            uptime_seconds = $stats.uptime_seconds,
            "Engine stats: {} orders, {} trades, {} volume, {} active, {}s uptime",
            $stats.total_orders,
            $stats.total_trades,
            $stats.total_volume,
            $stats.active_orders,
            $stats.uptime_seconds
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.console);
        assert!(!config.json_format);
        assert!(config.file.is_none());
    }

    #[test]
    fn test_logging_config_custom() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            file: Some("/tmp/test.log".to_string()),
            console: false,
            json_format: true,
        };

        assert_eq!(config.level, "debug");
        assert!(!config.console);
        assert!(config.json_format);
        assert!(config.file.is_some());
    }
}
