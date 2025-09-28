use crate::matching_engine::MatchingEngine;
use crate::types::*;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use chrono::Utc;
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

/// WebSocket 状态
#[derive(Clone)]
pub struct WebSocketState {
    pub engine: Arc<MatchingEngine>,
}

/// WebSocket 订阅类型
#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionType {
    Trades,
    OrderBook,
    MarketData,
    OrderUpdates,
    All,
}

/// WebSocket 连接信息
#[derive(Debug)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub subscriptions: Vec<SubscriptionType>,
    pub symbols: Vec<Symbol>,
}

impl ConnectionInfo {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            subscriptions: vec![SubscriptionType::All],
            symbols: vec![],
        }
    }
}

/// 创建 WebSocket 路由
pub fn create_websocket_router(engine: Arc<MatchingEngine>) -> Router {
    let state = WebSocketState { engine };

    Router::new()
        .route("/ws", get(websocket_handler))
        .route("/ws/trades", get(websocket_trades_handler))
        .route("/ws/orderbook", get(websocket_orderbook_handler))
        .route("/ws/market-data", get(websocket_market_data_handler))
        .with_state(state)
}

/// WebSocket 主处理器
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<WebSocketState>) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state, SubscriptionType::All))
}

/// WebSocket 交易数据处理器
async fn websocket_trades_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state, SubscriptionType::Trades))
}

/// WebSocket 订单簿数据处理器
async fn websocket_orderbook_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state, SubscriptionType::OrderBook))
}

/// WebSocket 市场数据处理器
async fn websocket_market_data_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state, SubscriptionType::MarketData))
}

/// WebSocket 连接处理
async fn websocket_connection(
    socket: WebSocket,
    state: WebSocketState,
    default_subscription: SubscriptionType,
) {
    let connection_info = ConnectionInfo::new();
    info!("WebSocket connection established: {}", connection_info.id);

    // 订阅广播通道
    let mut trade_receiver = state.engine.subscribe_trades();
    let mut order_receiver = state.engine.subscribe_orders();
    let mut market_data_receiver = state.engine.subscribe_market_data();

    let (mut sender, mut receiver) = socket.split();

    // 发送欢迎消息
    let welcome_msg = WebSocketMessage::Trade(Trade {
        id: Uuid::new_v4(),
        symbol: Symbol::new("SYSTEM", "WELCOME"),
        buy_order_id: Uuid::new_v4(),
        sell_order_id: Uuid::new_v4(),
        quantity: 0.0,
        price: 0.0,
        timestamp: Utc::now(),
        buyer_id: "system".to_string(),
        seller_id: "system".to_string(),
    });

    if let Ok(msg) = serde_json::to_string(&welcome_msg) {
        let _ = sender.send(Message::Text(msg)).await;
    }

    // 创建任务来处理不同的消息流
    let trade_task = tokio::spawn({
        let state = state.clone();
        let connection_info = connection_info.clone();
        async move {
            while let Ok(trade) = trade_receiver.recv().await {
                if should_send_trade(&connection_info, &trade) {
                    let msg = WebSocketMessage::Trade(trade);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    let order_task = tokio::spawn({
        let state = state.clone();
        let connection_info = connection_info.clone();
        async move {
            while let Ok(order) = order_receiver.recv().await {
                if should_send_order_update(&connection_info, &order) {
                    let msg = WebSocketMessage::OrderUpdate(order);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    let market_data_task = tokio::spawn({
        let state = state.clone();
        let connection_info = connection_info.clone();
        async move {
            while let Ok(market_data) = market_data_receiver.recv().await {
                if should_send_market_data(&connection_info, &market_data) {
                    let msg = WebSocketMessage::MarketData(market_data);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // 处理客户端消息
    let client_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received WebSocket message: {}", text);
                    // 这里可以处理客户端发送的订阅请求等
                    // 例如：{"type": "subscribe", "channel": "trades", "symbol": "BTCUSDT"}
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed: {}", connection_info.id);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if sender.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {
                    // 忽略 pong 消息
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // 等待任一任务完成
    tokio::select! {
        _ = trade_task => {},
        _ = order_task => {},
        _ = market_data_task => {},
        _ = client_task => {},
    }

    info!("WebSocket connection closed: {}", connection_info.id);
}

/// 检查是否应该发送交易数据
fn should_send_trade(connection_info: &ConnectionInfo, trade: &Trade) -> bool {
    if connection_info
        .subscriptions
        .contains(&SubscriptionType::All)
        || connection_info
            .subscriptions
            .contains(&SubscriptionType::Trades)
    {
        // 如果没有指定特定交易对，发送所有交易
        if connection_info.symbols.is_empty() {
            return true;
        }
        // 否则只发送指定交易对的交易
        connection_info.symbols.contains(&trade.symbol)
    } else {
        false
    }
}

/// 检查是否应该发送订单更新
fn should_send_order_update(connection_info: &ConnectionInfo, order: &Order) -> bool {
    if connection_info
        .subscriptions
        .contains(&SubscriptionType::All)
        || connection_info
            .subscriptions
            .contains(&SubscriptionType::OrderUpdates)
    {
        // 如果没有指定特定交易对，发送所有订单更新
        if connection_info.symbols.is_empty() {
            return true;
        }
        // 否则只发送指定交易对的订单更新
        connection_info.symbols.contains(&order.symbol)
    } else {
        false
    }
}

/// 检查是否应该发送市场数据
fn should_send_market_data(connection_info: &ConnectionInfo, market_data: &MarketData) -> bool {
    if connection_info
        .subscriptions
        .contains(&SubscriptionType::All)
        || connection_info
            .subscriptions
            .contains(&SubscriptionType::MarketData)
    {
        // 如果没有指定特定交易对，发送所有市场数据
        if connection_info.symbols.is_empty() {
            return true;
        }
        // 否则只发送指定交易对的市场数据
        connection_info.symbols.contains(&market_data.symbol)
    } else {
        false
    }
}

/// WebSocket 消息广播器
pub struct WebSocketBroadcaster {
    connections:
        Arc<tokio::sync::RwLock<HashMap<Uuid, tokio::sync::mpsc::UnboundedSender<Message>>>>,
}

impl WebSocketBroadcaster {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(
        &self,
        id: Uuid,
        sender: tokio::sync::mpsc::UnboundedSender<Message>,
    ) {
        let mut connections = self.connections.write().await;
        connections.insert(id, sender);
    }

    pub async fn remove_connection(&self, id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(&id);
    }

    pub async fn broadcast(&self, message: Message) {
        let connections = self.connections.read().await;
        let mut to_remove = Vec::new();

        for (id, sender) in connections.iter() {
            if sender.send(message.clone()).is_err() {
                to_remove.push(*id);
            }
        }

        // 移除失效的连接
        if !to_remove.is_empty() {
            drop(connections);
            let mut connections = self.connections.write().await;
            for id in to_remove {
                connections.remove(&id);
            }
        }
    }

    pub async fn broadcast_to_symbol(&self, message: Message, symbol: &Symbol) {
        // 这里可以实现更复杂的过滤逻辑
        // 目前简化处理，广播给所有连接
        self.broadcast(message).await;
    }
}

/// WebSocket 管理器
pub struct WebSocketManager {
    pub broadcaster: WebSocketBroadcaster,
    pub engine: Arc<MatchingEngine>,
}

impl WebSocketManager {
    pub fn new(engine: Arc<MatchingEngine>) -> Self {
        Self {
            broadcaster: WebSocketBroadcaster::new(),
            engine,
        }
    }

    pub async fn start_broadcasting(&self) {
        let mut trade_receiver = self.engine.subscribe_trades();
        let mut order_receiver = self.engine.subscribe_orders();
        let mut market_data_receiver = self.engine.subscribe_market_data();

        // 广播交易数据
        tokio::spawn({
            let broadcaster = self.broadcaster.clone();
            async move {
                while let Ok(trade) = trade_receiver.recv().await {
                    let msg = WebSocketMessage::Trade(trade);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        broadcaster.broadcast(Message::Text(json)).await;
                    }
                }
            }
        });

        // 广播订单更新
        tokio::spawn({
            let broadcaster = self.broadcaster.clone();
            async move {
                while let Ok(order) = order_receiver.recv().await {
                    let msg = WebSocketMessage::OrderUpdate(order);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        broadcaster.broadcast(Message::Text(json)).await;
                    }
                }
            }
        });

        // 广播市场数据
        tokio::spawn({
            let broadcaster = self.broadcaster.clone();
            async move {
                while let Ok(market_data) = market_data_receiver.recv().await {
                    let msg = WebSocketMessage::MarketData(market_data);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        broadcaster.broadcast(Message::Text(json)).await;
                    }
                }
            }
        });
    }
}

impl Clone for WebSocketBroadcaster {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_info() {
        let info = ConnectionInfo::new();
        assert_eq!(info.subscriptions, vec![SubscriptionType::All]);
        assert!(info.symbols.is_empty());
    }

    #[test]
    fn test_should_send_trade() {
        let mut info = ConnectionInfo::new();
        let trade = Trade {
            id: Uuid::new_v4(),
            symbol: Symbol::new("BTC", "USDT"),
            buy_order_id: Uuid::new_v4(),
            sell_order_id: Uuid::new_v4(),
            quantity: 1.0,
            price: 50000.0,
            timestamp: Utc::now(),
            buyer_id: "buyer".to_string(),
            seller_id: "seller".to_string(),
        };

        // 默认订阅所有
        assert!(should_send_trade(&info, &trade));

        // 只订阅交易
        info.subscriptions = vec![SubscriptionType::Trades];
        assert!(should_send_trade(&info, &trade));

        // 不订阅交易
        info.subscriptions = vec![SubscriptionType::OrderBook];
        assert!(!should_send_trade(&info, &trade));
    }
}
