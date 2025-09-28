use crate::config::MonitoringConfig;
use crate::types::*;
use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use metrics::{
    counter, gauge, histogram, register_counter, register_gauge, register_histogram, Counter,
    Gauge, Histogram,
};
use metrics_exporter_prometheus::PrometheusBuilder;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// 监控状态
#[derive(Clone)]
pub struct MonitoringState {
    pub config: MonitoringConfig,
    pub metrics: Arc<MatchingEngineMetrics>,
}

/// 撮合引擎指标
#[derive(Debug)]
pub struct MatchingEngineMetrics {
    // 订单相关指标
    pub orders_total: Counter,
    pub orders_filled: Counter,
    pub orders_cancelled: Counter,
    pub orders_rejected: Counter,
    pub active_orders: Gauge,

    // 交易相关指标
    pub trades_total: Counter,
    pub trade_volume_total: Counter,
    pub trade_volume_24h: Gauge,

    // 性能指标
    pub order_processing_duration: Histogram,
    pub trade_execution_duration: Histogram,
    pub orderbook_update_duration: Histogram,

    // 系统指标
    pub memory_usage: Gauge,
    pub cpu_usage: Gauge,
    pub uptime_seconds: Gauge,

    // 业务指标
    pub spread_avg: Gauge,
    pub spread_max: Gauge,
    pub spread_min: Gauge,
    pub orderbook_depth: Gauge,

    // 错误指标
    pub errors_total: Counter,
    pub websocket_connections: Gauge,
    pub api_requests_total: Counter,
    pub api_request_duration: Histogram,
}

impl MatchingEngineMetrics {
    pub fn new() -> Self {
        Self {
            orders_total: register_counter!(
                "matching_engine_orders_total",
                "Total number of orders"
            ),
            orders_filled: register_counter!(
                "matching_engine_orders_filled_total",
                "Total number of filled orders"
            ),
            orders_cancelled: register_counter!(
                "matching_engine_orders_cancelled_total",
                "Total number of cancelled orders"
            ),
            orders_rejected: register_counter!(
                "matching_engine_orders_rejected_total",
                "Total number of rejected orders"
            ),
            active_orders: register_gauge!(
                "matching_engine_active_orders",
                "Number of active orders"
            ),

            trades_total: register_counter!(
                "matching_engine_trades_total",
                "Total number of trades"
            ),
            trade_volume_total: register_counter!(
                "matching_engine_trade_volume_total",
                "Total trade volume"
            ),
            trade_volume_24h: register_gauge!(
                "matching_engine_trade_volume_24h",
                "24-hour trade volume"
            ),

            order_processing_duration: register_histogram!(
                "matching_engine_order_processing_duration_seconds",
                "Order processing duration"
            ),
            trade_execution_duration: register_histogram!(
                "matching_engine_trade_execution_duration_seconds",
                "Trade execution duration"
            ),
            orderbook_update_duration: register_histogram!(
                "matching_engine_orderbook_update_duration_seconds",
                "Orderbook update duration"
            ),

            memory_usage: register_gauge!(
                "matching_engine_memory_usage_bytes",
                "Memory usage in bytes"
            ),
            cpu_usage: register_gauge!("matching_engine_cpu_usage_percent", "CPU usage percentage"),
            uptime_seconds: register_gauge!(
                "matching_engine_uptime_seconds",
                "Engine uptime in seconds"
            ),

            spread_avg: register_gauge!("matching_engine_spread_avg", "Average spread"),
            spread_max: register_gauge!("matching_engine_spread_max", "Maximum spread"),
            spread_min: register_gauge!("matching_engine_spread_min", "Minimum spread"),
            orderbook_depth: register_gauge!("matching_engine_orderbook_depth", "Orderbook depth"),

            errors_total: register_counter!(
                "matching_engine_errors_total",
                "Total number of errors"
            ),
            websocket_connections: register_gauge!(
                "matching_engine_websocket_connections",
                "Number of WebSocket connections"
            ),
            api_requests_total: register_counter!(
                "matching_engine_api_requests_total",
                "Total number of API requests"
            ),
            api_request_duration: register_histogram!(
                "matching_engine_api_request_duration_seconds",
                "API request duration"
            ),
        }
    }
}

/// 监控管理器
pub struct MonitoringManager {
    pub config: MonitoringConfig,
    pub metrics: Arc<MatchingEngineMetrics>,
    pub start_time: Instant,
    pub stats_cache: Arc<RwLock<HashMap<String, f64>>>,
}

impl MonitoringManager {
    pub fn new(config: MonitoringConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // 初始化 Prometheus 指标导出器
        let builder = PrometheusBuilder::new();
        let (recorder, exporter) = builder
            .with_http_listener(([0, 0, 0, 0], config.metrics_port))
            .build()?;

        // 设置全局指标记录器
        metrics::set_boxed_recorder(Box::new(recorder))?;

        // 启动指标导出器
        tokio::spawn(async move {
            if let Err(e) = exporter.await {
                error!("Prometheus exporter error: {}", e);
            }
        });

        info!(
            "Monitoring system initialized on port {}",
            config.metrics_port
        );

        Ok(Self {
            config,
            metrics: Arc::new(MatchingEngineMetrics::new()),
            start_time: Instant::now(),
            stats_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 记录订单提交
    pub fn record_order_submitted(&self, order: &Order) {
        counter!(self.metrics.orders_total, 1.0);
        gauge!(self.metrics.active_orders, 1.0);

        // 按交易对记录
        let labels = [("symbol", order.symbol.to_string())];
        counter!(self.metrics.orders_total, 1.0, &labels);
    }

    /// 记录订单成交
    pub fn record_order_filled(&self, order: &Order) {
        counter!(self.metrics.orders_filled, 1.0);
        gauge!(self.metrics.active_orders, -1.0);

        let labels = [("symbol", order.symbol.to_string())];
        counter!(self.metrics.orders_filled, 1.0, &labels);
    }

    /// 记录订单取消
    pub fn record_order_cancelled(&self, order: &Order) {
        counter!(self.metrics.orders_cancelled, 1.0);
        gauge!(self.metrics.active_orders, -1.0);

        let labels = [("symbol", order.symbol.to_string())];
        counter!(self.metrics.orders_cancelled, 1.0, &labels);
    }

    /// 记录订单拒绝
    pub fn record_order_rejected(&self, order: &Order, reason: &str) {
        counter!(self.metrics.orders_rejected, 1.0);

        let labels = [
            ("symbol", order.symbol.to_string()),
            ("reason", reason.to_string()),
        ];
        counter!(self.metrics.orders_rejected, 1.0, &labels);
    }

    /// 记录交易执行
    pub fn record_trade_executed(&self, trade: &Trade) {
        counter!(self.metrics.trades_total, 1.0);
        counter!(
            self.metrics.trade_volume_total,
            trade.quantity * trade.price
        );

        let labels = [("symbol", trade.symbol.to_string())];
        counter!(self.metrics.trades_total, 1.0, &labels);
        counter!(
            self.metrics.trade_volume_total,
            trade.quantity * trade.price,
            &labels
        );
    }

    /// 记录订单处理时间
    pub fn record_order_processing_time(&self, duration: Duration) {
        histogram!(
            self.metrics.order_processing_duration,
            duration.as_secs_f64()
        );
    }

    /// 记录交易执行时间
    pub fn record_trade_execution_time(&self, duration: Duration) {
        histogram!(
            self.metrics.trade_execution_duration,
            duration.as_secs_f64()
        );
    }

    /// 记录订单簿更新时间
    pub fn record_orderbook_update_time(&self, duration: Duration) {
        histogram!(
            self.metrics.orderbook_update_duration,
            duration.as_secs_f64()
        );
    }

    /// 记录错误
    pub fn record_error(&self, error_type: &str, context: &str) {
        let labels = [
            ("error_type", error_type.to_string()),
            ("context", context.to_string()),
        ];
        counter!(self.metrics.errors_total, 1.0, &labels);
    }

    /// 记录API请求
    pub fn record_api_request(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        duration: Duration,
    ) {
        let labels = [
            ("method", method.to_string()),
            ("path", path.to_string()),
            ("status", status_code.to_string()),
        ];
        counter!(self.metrics.api_requests_total, 1.0, &labels);
        histogram!(
            self.metrics.api_request_duration,
            duration.as_secs_f64(),
            &labels
        );
    }

    /// 更新WebSocket连接数
    pub fn update_websocket_connections(&self, count: i64) {
        gauge!(self.metrics.websocket_connections, count as f64);
    }

    /// 更新系统指标
    pub async fn update_system_metrics(&self) {
        // 更新运行时间
        let uptime = self.start_time.elapsed().as_secs() as f64;
        gauge!(self.metrics.uptime_seconds, uptime);

        // 更新内存使用情况
        if let Ok(memory_usage) = get_memory_usage() {
            gauge!(self.metrics.memory_usage, memory_usage);
        }

        // 更新CPU使用情况
        if let Ok(cpu_usage) = get_cpu_usage().await {
            gauge!(self.metrics.cpu_usage, cpu_usage);
        }
    }

    /// 更新业务指标
    pub async fn update_business_metrics(
        &self,
        stats: &EngineStats,
        market_data: &HashMap<Symbol, MarketData>,
    ) {
        // 更新24小时交易量
        let total_volume_24h: f64 = market_data.values().map(|data| data.volume_24h).sum();
        gauge!(self.metrics.trade_volume_24h, total_volume_24h);

        // 更新价差指标
        let spreads: Vec<f64> = market_data
            .values()
            .filter_map(|data| {
                // 这里需要从订单簿获取价差，简化处理
                Some(0.0) // 实际实现中应该计算真实价差
            })
            .collect();

        if !spreads.is_empty() {
            let avg_spread = spreads.iter().sum::<f64>() / spreads.len() as f64;
            let max_spread = spreads.iter().fold(0.0, |a, &b| a.max(b));
            let min_spread = spreads.iter().fold(f64::INFINITY, |a, &b| a.min(b));

            gauge!(self.metrics.spread_avg, avg_spread);
            gauge!(self.metrics.spread_max, max_spread);
            gauge!(self.metrics.spread_min, min_spread);
        }

        // 更新订单簿深度
        gauge!(self.metrics.orderbook_depth, stats.active_orders as f64);
    }

    /// 获取指标数据
    pub async fn get_metrics(&self) -> String {
        // 这里应该返回 Prometheus 格式的指标数据
        // 由于我们使用了 metrics-exporter-prometheus，它会自动处理
        "".to_string()
    }
}

/// 获取内存使用情况
fn get_memory_usage() -> Result<f64, Box<dyn std::error::Error>> {
    // 简化实现，实际应该使用系统API
    Ok(0.0)
}

/// 获取CPU使用情况
async fn get_cpu_usage() -> Result<f64, Box<dyn std::error::Error>> {
    // 简化实现，实际应该使用系统API
    Ok(0.0)
}

/// 创建监控路由
pub fn create_monitoring_router(config: MonitoringConfig) -> Router {
    let state = MonitoringState {
        config: config.clone(),
        metrics: Arc::new(MatchingEngineMetrics::new()),
    };

    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/stats", get(get_stats))
        .with_state(state)
}

/// 健康检查
async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}

/// 获取指标
async fn get_metrics(State(state): State<MonitoringState>) -> Result<String, StatusCode> {
    // 这里应该返回 Prometheus 格式的指标
    // 由于我们使用了 metrics-exporter-prometheus，它会自动处理
    Ok("".to_string())
}

/// 获取统计信息
async fn get_stats(
    State(state): State<MonitoringState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "metrics_enabled": state.config.enabled,
        "metrics_port": state.config.metrics_port,
        "performance_metrics": state.config.enable_performance_metrics,
        "business_metrics": state.config.enable_business_metrics
    })))
}

/// 性能计时器
pub struct PerformanceTimer {
    start_time: Instant,
    metric: &'static str,
}

impl PerformanceTimer {
    pub fn start(metric: &'static str) -> Self {
        Self {
            start_time: Instant::now(),
            metric,
        }
    }

    pub fn finish(self) -> Duration {
        let duration = self.start_time.elapsed();
        info!(
            "Performance: {} took {}ms",
            self.metric,
            duration.as_millis()
        );
        duration
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        info!(
            "Performance: {} took {}ms",
            self.metric,
            duration.as_millis()
        );
    }
}

/// 性能计时宏
#[macro_export]
macro_rules! time_it {
    ($metric:expr, $code:block) => {{
        let _timer = $crate::monitoring::PerformanceTimer::start($metric);
        $code
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = MatchingEngineMetrics::new();
        // 测试指标是否正确创建
        assert!(metrics.orders_total.name().contains("orders_total"));
        assert!(metrics.trades_total.name().contains("trades_total"));
    }

    #[test]
    fn test_performance_timer() {
        let timer = PerformanceTimer::start("test_metric");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = timer.finish();
        assert!(duration.as_millis() >= 10);
    }
}
