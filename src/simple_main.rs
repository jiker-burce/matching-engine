use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

use crate::matching_engine::MatchingEngine;

/// 简化的 API 状态
#[derive(Clone)]
pub struct SimpleApiState {
    pub engine: Arc<MatchingEngine>,
}

/// 创建简化的路由
pub fn create_simple_router(engine: Arc<MatchingEngine>) -> Router {
    let state = SimpleApiState { engine };

    Router::new()
        .route("/health", get(health_check))
        .route("/stats", get(get_engine_stats))
        .with_state(state)
}

/// 健康检查
async fn health_check(
    State(state): State<SimpleApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = state.engine.get_stats();

    Ok(Json(json!({
        "status": "healthy",
        "uptime_seconds": stats.uptime_seconds,
        "total_orders": stats.total_orders,
        "total_trades": stats.total_trades,
        "active_orders": stats.active_orders
    })))
}

/// 获取引擎统计信息
async fn get_engine_stats(
    State(state): State<SimpleApiState>,
) -> Result<Json<crate::types::EngineStats>, StatusCode> {
    Ok(Json(state.engine.get_stats()))
}

/// 简化的主函数
pub async fn run_simple_server() -> Result<()> {
    // 初始化简单的日志
    tracing_subscriber::fmt::init();

    info!(
        "Starting Simple Matching Engine v{}",
        env!("CARGO_PKG_VERSION")
    );

    // 创建撮合引擎
    let engine = Arc::new(MatchingEngine::new());
    info!("Matching engine initialized");

    // 创建路由
    let app = create_simple_router(engine);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await?;
    info!("Server listening on 0.0.0.0:8888");

    // 启动服务器
    axum::serve(listener, app).await?;

    Ok(())
}
