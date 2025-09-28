use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, warn};

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 服务器配置
    pub server: ServerConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 监控配置
    pub monitoring: MonitoringConfig,
    /// 撮合引擎配置
    pub engine: EngineConfig,
    /// 数据库配置（预留）
    pub database: Option<DatabaseConfig>,
    /// Redis配置（预留）
    pub redis: Option<RedisConfig>,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// API路径前缀
    pub api_prefix: String,
    /// WebSocket路径前缀
    pub ws_prefix: String,
    /// CORS配置
    pub cors: CorsConfig,
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    /// 最大请求体大小（字节）
    pub max_request_size: usize,
}

/// CORS配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// 允许的源
    pub allowed_origins: Vec<String>,
    /// 允许的方法
    pub allowed_methods: Vec<String>,
    /// 允许的头部
    pub allowed_headers: Vec<String>,
    /// 是否允许凭据
    pub allow_credentials: bool,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 日志文件路径
    pub file: Option<String>,
    /// 是否输出到控制台
    pub console: bool,
    /// 是否使用JSON格式
    pub json_format: bool,
    /// 日志轮转配置
    pub rotation: LogRotationConfig,
}

/// 日志轮转配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// 轮转策略：daily, hourly, size
    pub strategy: String,
    /// 最大文件大小（字节）
    pub max_size: Option<u64>,
    /// 保留天数
    pub max_age: Option<u64>,
    /// 最大文件数量
    pub max_files: Option<u32>,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 是否启用监控
    pub enabled: bool,
    /// Prometheus指标端口
    pub metrics_port: u16,
    /// 指标路径
    pub metrics_path: String,
    /// 健康检查路径
    pub health_path: String,
    /// 是否启用性能指标
    pub enable_performance_metrics: bool,
    /// 是否启用业务指标
    pub enable_business_metrics: bool,
}

/// 撮合引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// 最大订单数量
    pub max_orders: u64,
    /// 最大交易历史数量
    pub max_trades: u64,
    /// 订单簿最大深度
    pub max_orderbook_depth: usize,
    /// 是否启用价格保护
    pub enable_price_protection: bool,
    /// 最大价格偏差百分比
    pub max_price_deviation: f64,
    /// 是否启用交易限制
    pub enable_trade_limits: bool,
    /// 单笔最大交易量
    pub max_trade_quantity: f64,
    /// 单日最大交易量
    pub max_daily_volume: f64,
    /// 支持的交易对
    pub supported_symbols: Vec<String>,
}

/// 数据库配置（预留）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
}

/// Redis配置（预留）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub command_timeout: u64,
}

impl AppConfig {
    /// 从配置文件加载配置
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            // 默认配置
            .add_source(File::with_name("config/default").required(false))
            // 环境特定配置
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // 本地配置（不提交到版本控制）
            .add_source(File::with_name("config/local").required(false))
            // 环境变量
            .add_source(Environment::with_prefix("MATCHING_ENGINE").separator("_"))
            .build()?;

        let app_config: AppConfig = config.try_deserialize()?;

        info!("Configuration loaded for mode: {}", run_mode);
        info!(
            "Server: {}:{}",
            app_config.server.host, app_config.server.port
        );
        info!("Log level: {}", app_config.logging.level);
        info!("Monitoring enabled: {}", app_config.monitoring.enabled);

        Ok(app_config)
    }

    /// 获取服务器地址
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// 获取API基础URL
    pub fn api_base_url(&self) -> String {
        format!("http://{}/{}", self.server_addr(), self.server.api_prefix)
    }

    /// 获取WebSocket基础URL
    pub fn ws_base_url(&self) -> String {
        format!("ws://{}/{}", self.server_addr(), self.server.ws_prefix)
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        // 验证服务器配置
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        if self.server.request_timeout == 0 {
            return Err("Request timeout cannot be 0".to_string());
        }

        // 验证日志配置
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(format!("Invalid log level: {}", self.logging.level));
        }

        // 验证监控配置
        if self.monitoring.enabled && self.monitoring.metrics_port == 0 {
            return Err("Metrics port cannot be 0 when monitoring is enabled".to_string());
        }

        // 验证引擎配置
        if self.engine.max_orders == 0 {
            return Err("Max orders cannot be 0".to_string());
        }

        if self.engine.max_price_deviation < 0.0 || self.engine.max_price_deviation > 100.0 {
            return Err("Max price deviation must be between 0 and 100".to_string());
        }

        if self.engine.max_trade_quantity <= 0.0 {
            return Err("Max trade quantity must be positive".to_string());
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            monitoring: MonitoringConfig::default(),
            engine: EngineConfig::default(),
            database: None,
            redis: None,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            api_prefix: "api/v1".to_string(),
            ws_prefix: "ws".to_string(),
            cors: CorsConfig::default(),
            request_timeout: 30,
            max_request_size: 1024 * 1024, // 1MB
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            allow_credentials: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
            console: true,
            json_format: false,
            rotation: LogRotationConfig::default(),
        }
    }
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            strategy: "daily".to_string(),
            max_size: Some(100 * 1024 * 1024), // 100MB
            max_age: Some(30),                 // 30 days
            max_files: Some(10),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_port: 9090,
            metrics_path: "/metrics".to_string(),
            health_path: "/health".to_string(),
            enable_performance_metrics: true,
            enable_business_metrics: true,
        }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_orders: 1_000_000,
            max_trades: 10_000_000,
            max_orderbook_depth: 1000,
            enable_price_protection: true,
            max_price_deviation: 10.0, // 10%
            enable_trade_limits: true,
            max_trade_quantity: 1000.0,
            max_daily_volume: 1_000_000.0,
            supported_symbols: vec![
                "BTCUSDT".to_string(),
                "ETHUSDT".to_string(),
                "BNBUSDT".to_string(),
            ],
        }
    }
}

/// 配置构建器
pub struct ConfigBuilder {
    config: AppConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    pub fn server(mut self, server: ServerConfig) -> Self {
        self.config.server = server;
        self
    }

    pub fn logging(mut self, logging: LoggingConfig) -> Self {
        self.config.logging = logging;
        self
    }

    pub fn monitoring(mut self, monitoring: MonitoringConfig) -> Self {
        self.config.monitoring = monitoring;
        self
    }

    pub fn engine(mut self, engine: EngineConfig) -> Self {
        self.config.engine = engine;
        self
    }

    pub fn database(mut self, database: DatabaseConfig) -> Self {
        self.config.database = Some(database);
        self
    }

    pub fn redis(mut self, redis: RedisConfig) -> Self {
        self.config.redis = Some(redis);
        self
    }

    pub fn build(self) -> Result<AppConfig, String> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.logging.level, "info");
        assert!(config.monitoring.enabled);
        assert!(config.engine.enable_price_protection);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        // 测试无效端口
        config.server.port = 0;
        assert!(config.validate().is_err());

        // 测试无效日志级别
        config.server.port = 8080;
        config.logging.level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .server(ServerConfig {
                port: 9090,
                ..Default::default()
            })
            .build()
            .unwrap();

        assert_eq!(config.server.port, 9090);
    }

    #[test]
    fn test_server_addr() {
        let config = AppConfig::default();
        assert_eq!(config.server_addr(), "0.0.0.0:8080");
    }
}
