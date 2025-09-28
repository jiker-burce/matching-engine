use sqlx::{PgPool, Pool, Postgres};
use std::env;
use tracing::{error, info};

/// 数据库连接配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            database: env::var("DB_NAME").unwrap_or_else(|_| "trading_engine".to_string()),
            username: env::var("DB_USER").unwrap_or_else(|_| "trading_user".to_string()),
            password: env::var("DB_PASSWORD").unwrap_or_else(|_| "trading_password".to_string()),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        }
    }
}

impl DatabaseConfig {
    /// 构建数据库连接URL
    pub fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    /// 创建数据库连接池
    pub async fn create_pool(&self) -> Result<PgPool, sqlx::Error> {
        let url = self.connection_url();

        info!("Connecting to database: {}", self.host);

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(&url)
            .await?;

        // 测试连接
        sqlx::query("SELECT 1").fetch_one(&pool).await?;

        info!("Database connection established successfully");
        Ok(pool)
    }
}

/// 数据库连接管理器
pub struct DatabaseManager {
    pub pool: PgPool,
}

impl DatabaseManager {
    /// 创建新的数据库管理器
    pub async fn new(config: DatabaseConfig) -> Result<Self, sqlx::Error> {
        let pool = config.create_pool().await?;
        Ok(Self { pool })
    }

    /// 获取连接池
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }

    /// 获取数据库统计信息
    pub async fn get_stats(&self) -> Result<DatabaseStats, sqlx::Error> {
        let stats = sqlx::query_as!(
            DatabaseStats,
            r#"
            SELECT 
                (SELECT COUNT(*) FROM users) as total_users,
                (SELECT COUNT(*) FROM orders) as total_orders,
                (SELECT COUNT(*) FROM trades) as total_trades,
                (SELECT COUNT(*) FROM accounts) as total_accounts,
                (SELECT COUNT(*) FROM trading_pairs) as total_trading_pairs
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }
}

/// 数据库统计信息
#[derive(Debug)]
pub struct DatabaseStats {
    pub total_users: i64,
    pub total_orders: i64,
    pub total_trades: i64,
    pub total_accounts: i64,
    pub total_trading_pairs: i64,
}

/// 数据库迁移
pub struct DatabaseMigration;

impl DatabaseMigration {
    /// 运行数据库迁移
    pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
        info!("Running database migrations...");

        // 检查是否需要创建扩展
        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
            .execute(pool)
            .await?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"timescaledb\"")
            .execute(pool)
            .await?;

        // 这里可以添加更多的迁移逻辑
        // 例如：创建表、索引、视图等

        info!("Database migrations completed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let config = DatabaseConfig::default();
        let manager = DatabaseManager::new(config).await;

        match manager {
            Ok(manager) => {
                let health = manager.health_check().await;
                assert!(health.is_ok());
            }
            Err(e) => {
                // 在测试环境中，数据库可能不可用
                println!("Database connection test skipped: {}", e);
            }
        }
    }
}
