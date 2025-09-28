use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::matching_engine::MatchingEngine;

/// 简化的 API 状态
#[derive(Clone)]
pub struct SimpleApiState {
    pub engine: Arc<MatchingEngine>,
    pub trade_sender: broadcast::Sender<String>,
}

/// 创建简化的路由
pub fn create_simple_router(
    engine: Arc<MatchingEngine>,
    trade_sender: broadcast::Sender<String>,
) -> Router {
    let state = SimpleApiState {
        engine,
        trade_sender,
    };

    Router::new()
        .route("/health", get(health_check))
        .route("/stats", get(get_engine_stats))
        .route("/ws", get(websocket_handler))
        .route("/submit_order", post(submit_order_handler))
        .route("/orderbook/:symbol", get(get_orderbook))
        .route("/trades/:symbol", get(get_trades))
        .route("/market_data/:symbol", get(get_market_data))
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

/// WebSocket处理器
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SimpleApiState>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

/// WebSocket连接处理
async fn websocket_connection(socket: WebSocket, state: SimpleApiState) {
    let mut rx = state.trade_sender.subscribe();

    let (mut sender, mut receiver) = socket.split();

    // 发送连接成功消息
    let _ = sender
        .send(Message::Text(
            json!({
                "type": "connected",
                "message": "WebSocket连接成功"
            })
            .to_string(),
        ))
        .await;

    // 监听广播消息
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = sender.send(Message::Text(msg)).await {
                error!("WebSocket发送失败: {}", e);
                break;
            }
        }
    });

    // 处理接收到的消息
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("收到WebSocket消息: {}", text);
                // 这里可以处理客户端发送的消息
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket连接关闭");
                break;
            }
            Err(e) => {
                error!("WebSocket错误: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// 提交订单处理器
async fn submit_order_handler(
    State(state): State<SimpleApiState>,
    Json(_order_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 创建测试订单
    let order = crate::types::Order::new(
        crate::types::Symbol::new("BTC", "USDT"),
        crate::types::OrderSide::Buy,
        crate::types::OrderType::Limit,
        1.0,
        Some(45000.0),
        "test_user".to_string(),
    );

    match state.engine.submit_order(order).await {
        Ok(trades) => {
            // 广播交易信息
            let trade_msg = json!({
                "type": "trade",
                "trades": trades
            });
            let _ = state.trade_sender.send(trade_msg.to_string());

            Ok(Json(json!({
                "success": true,
                "message": format!("订单提交成功，执行了{}笔交易", trades.len()),
                "trades": trades
            })))
        }
        Err(e) => {
            error!("订单提交失败: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": e
            })))
        }
    }
}

/// 获取订单簿
async fn get_orderbook(
    Path(symbol): Path<String>,
    State(_state): State<SimpleApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 生成模拟订单簿数据
    let mock_orderbook = generate_mock_orderbook(&symbol);
    Ok(Json(mock_orderbook))
}

/// 获取交易历史
async fn get_trades(
    Path(symbol): Path<String>,
    State(_state): State<SimpleApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 生成模拟交易数据
    let mock_trades = generate_mock_trades(&symbol);
    Ok(Json(mock_trades))
}

/// 获取市场数据
async fn get_market_data(
    Path(symbol): Path<String>,
    State(_state): State<SimpleApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 生成模拟市场数据
    let mock_market_data = generate_mock_market_data(&symbol);
    Ok(Json(mock_market_data))
}

/// 生成模拟订单簿数据
fn generate_mock_orderbook(symbol: &str) -> serde_json::Value {
    let base_price = 45000.0;
    let mut bids = Vec::new();
    let mut asks = Vec::new();

    // 生成买盘数据（价格从高到低）
    for i in 0..10 {
        let price = base_price - (i + 1) as f64 * 10.0;
        let quantity = 0.1 + (i as f64 * 0.1);
        bids.push(json!({
            "price": price,
            "quantity": quantity,
            "total": price * quantity
        }));
    }

    // 生成卖盘数据（价格从低到高）
    for i in 0..10 {
        let price = base_price + (i + 1) as f64 * 10.0;
        let quantity = 0.1 + (i as f64 * 0.1);
        asks.push(json!({
            "price": price,
            "quantity": quantity,
            "total": price * quantity
        }));
    }

    json!({
        "symbol": symbol,
        "bids": bids,
        "asks": asks,
        "timestamp": Utc::now().to_rfc3339()
    })
}

/// 生成模拟交易数据
fn generate_mock_trades(_symbol: &str) -> serde_json::Value {
    let base_price = 45000.0;
    let mut trades = Vec::new();

    for i in 0..20 {
        let price = base_price + (i as f64 - 10.0) * 50.0;
        let quantity = 0.1 + (i as f64 * 0.05);
        let side = if i % 2 == 0 { "buy" } else { "sell" };

        trades.push(json!({
            "id": format!("trade_{}_{}", Utc::now().timestamp(), i),
            "price": price,
            "quantity": quantity,
            "side": side,
            "timestamp": Utc::now().to_rfc3339()
        }));
    }

    json!(trades)
}

/// 生成模拟市场数据
fn generate_mock_market_data(symbol: &str) -> serde_json::Value {
    let base_price = 45000.0;

    json!({
        "symbol": symbol,
        "price": base_price,
        "price_change_24h": 1200.0,
        "price_change_percentage_24h": 2.73,
        "total_volume": 25000000000.0,
        "high_24h": base_price * 1.05,
        "low_24h": base_price * 0.95,
        "timestamp": Utc::now().to_rfc3339()
    })
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

    // 创建广播通道
    let (trade_sender, _) = broadcast::channel(1000);
    info!("WebSocket broadcast channel created");

    // 创建路由
    let app = create_simple_router(engine, trade_sender);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await?;
    info!("Server listening on 0.0.0.0:8888");
    info!("WebSocket endpoint: ws://localhost:8888/ws");

    // 启动服务器
    axum::serve(listener, app).await?;

    Ok(())
}
