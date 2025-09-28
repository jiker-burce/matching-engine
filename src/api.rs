use crate::matching_engine::MatchingEngine;
use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// API 状态
#[derive(Clone)]
pub struct ApiState {
    pub engine: Arc<MatchingEngine>,
}

/// 创建 API 路由
pub fn create_router(engine: Arc<MatchingEngine>) -> Router {
    let state = ApiState { engine };

    Router::new()
        .route("/health", get(health_check))
        .route("/stats", get(get_engine_stats))
        .route("/orders", post(create_order))
        .route("/orders/:order_id", get(get_order))
        .route("/orders/:order_id", delete(cancel_order))
        .route("/orders/user/:user_id", get(get_user_orders))
        .route("/orderbook/:symbol", get(get_orderbook))
        .route("/market-data", get(get_all_market_data))
        .route("/market-data/:symbol", get(get_market_data))
        .route("/trades", get(get_trades))
        .route("/trades/:symbol", get(get_symbol_trades))
        .with_state(state)
}

/// 健康检查
async fn health_check(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
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
async fn get_engine_stats(State(state): State<ApiState>) -> Result<Json<EngineStats>, StatusCode> {
    Ok(Json(state.engine.get_stats()))
}

/// 创建订单
async fn create_order(
    State(state): State<ApiState>,
    Json(request): Json<CreateOrderRequest>,
) -> Result<Json<CreateOrderResponse>, StatusCode> {
    info!("Creating order for user {}: {:?}", request.user_id, request);

    let order = Order::new(
        request.symbol,
        request.side,
        request.order_type,
        request.quantity,
        request.price,
        request.user_id.clone(),
    );

    match state.engine.submit_order(order.clone()).await {
        Ok(trades) => {
            info!(
                "Order {} created successfully, {} trades executed",
                order.id,
                trades.len()
            );

            let status = if trades.is_empty() {
                OrderStatus::New
            } else if order.remaining_quantity > 0.0 {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Filled
            };

            Ok(Json(CreateOrderResponse {
                order_id: order.id,
                status,
                message: format!(
                    "Order created successfully, {} trades executed",
                    trades.len()
                ),
            }))
        }
        Err(e) => {
            error!("Failed to create order: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 获取订单信息
async fn get_order(
    State(state): State<ApiState>,
    Path(order_id): Path<String>,
) -> Result<Json<Order>, StatusCode> {
    let order_id = match Uuid::parse_str(&order_id) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    match state.engine.get_order(order_id) {
        Some(order) => Ok(Json(order)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// 取消订单
async fn cancel_order(
    State(state): State<ApiState>,
    Path(order_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<CancelOrderResponse>, StatusCode> {
    let order_id = match Uuid::parse_str(&order_id) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let user_id = match params.get("user_id") {
        Some(id) => id.clone(),
        None => return Err(StatusCode::BAD_REQUEST),
    };

    match state.engine.cancel_order(order_id, user_id).await {
        Ok(_) => Ok(Json(CancelOrderResponse {
            success: true,
            message: "Order cancelled successfully".to_string(),
        })),
        Err(e) => {
            warn!("Failed to cancel order {}: {}", order_id, e);
            Ok(Json(CancelOrderResponse {
                success: false,
                message: e,
            }))
        }
    }
}

/// 获取用户订单
async fn get_user_orders(
    State(state): State<ApiState>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Order>>, StatusCode> {
    let orders = state.engine.get_user_orders(&user_id);
    Ok(Json(orders))
}

/// 获取订单簿深度
async fn get_orderbook(
    State(state): State<ApiState>,
    Path(symbol_str): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<OrderBookDepth>, StatusCode> {
    // 解析交易对符号
    let symbol = parse_symbol(&symbol_str)?;

    let depth = params.get("depth").and_then(|d| d.parse::<usize>().ok());

    match state.engine.get_orderbook_depth(&symbol, depth) {
        Some(orderbook) => Ok(Json(orderbook)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// 获取所有市场数据
async fn get_all_market_data(
    State(state): State<ApiState>,
) -> Result<Json<HashMap<Symbol, MarketData>>, StatusCode> {
    Ok(Json(state.engine.get_all_market_data()))
}

/// 获取特定交易对的市场数据
async fn get_market_data(
    State(state): State<ApiState>,
    Path(symbol_str): Path<String>,
) -> Result<Json<MarketData>, StatusCode> {
    let symbol = parse_symbol(&symbol_str)?;

    match state.engine.get_market_data(&symbol) {
        Some(market_data) => Ok(Json(market_data)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// 获取交易历史
async fn get_trades(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Trade>>, StatusCode> {
    let limit = params.get("limit").and_then(|l| l.parse::<usize>().ok());

    let trades = state.engine.get_trades(None, limit);
    Ok(Json(trades))
}

/// 获取特定交易对的交易历史
async fn get_symbol_trades(
    State(state): State<ApiState>,
    Path(symbol_str): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Trade>>, StatusCode> {
    let symbol = parse_symbol(&symbol_str)?;
    let limit = params.get("limit").and_then(|l| l.parse::<usize>().ok());

    let trades = state.engine.get_trades(Some(&symbol), limit);
    Ok(Json(trades))
}

/// 解析交易对符号
fn parse_symbol(symbol_str: &str) -> Result<Symbol, StatusCode> {
    // 支持格式: BTCUSDT, BTC-USDT, BTC/USDT
    let parts: Vec<&str> = if symbol_str.contains('-') {
        symbol_str.split('-').collect()
    } else if symbol_str.contains('/') {
        symbol_str.split('/').collect()
    } else {
        // 假设是 BTCUSDT 格式，需要智能分割
        // 这里简化处理，假设前3个字符是基础货币
        if symbol_str.len() >= 6 {
            vec![&symbol_str[..3], &symbol_str[3..]]
        } else {
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if parts.len() != 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Symbol::new(parts[0], parts[1]))
}

/// 错误响应
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// 将错误转换为 JSON 响应
pub fn error_response(error: &str, message: &str) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_symbol() {
        assert_eq!(parse_symbol("BTCUSDT").unwrap(), Symbol::new("BTC", "USDT"));
        assert_eq!(
            parse_symbol("BTC-USDT").unwrap(),
            Symbol::new("BTC", "USDT")
        );
        assert_eq!(
            parse_symbol("BTC/USDT").unwrap(),
            Symbol::new("BTC", "USDT")
        );
        assert_eq!(parse_symbol("ETHUSDT").unwrap(), Symbol::new("ETH", "USDT"));
    }

    #[test]
    fn test_parse_symbol_invalid() {
        assert!(parse_symbol("INVALID").is_err());
        assert!(parse_symbol("").is_err());
        assert!(parse_symbol("BTC").is_err());
    }
}
