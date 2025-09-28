use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    /// 限价单
    Limit,
    /// 市价单
    Market,
    /// 止损单
    StopLoss,
    /// 止盈单
    TakeProfit,
}

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    /// 买入
    Buy,
    /// 卖出
    Sell,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// 新订单
    New,
    /// 部分成交
    PartiallyFilled,
    /// 完全成交
    Filled,
    /// 已取消
    Cancelled,
    /// 已拒绝
    Rejected,
}

/// 交易对
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub base: String,  // 基础货币，如 BTC
    pub quote: String, // 计价货币，如 USDT
}

impl Symbol {
    pub fn new(base: &str, quote: &str) -> Self {
        Self {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>, // 市价单可能没有价格
    pub status: OrderStatus,
    pub filled_quantity: f64,
    pub remaining_quantity: f64,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
}

impl Order {
    pub fn new(
        symbol: Symbol,
        side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
        user_id: String,
    ) -> Self {
        let id = Uuid::new_v4();
        let timestamp = Utc::now();

        Self {
            id,
            symbol,
            side,
            order_type,
            quantity,
            price,
            status: OrderStatus::New,
            filled_quantity: 0.0,
            remaining_quantity: quantity,
            timestamp,
            user_id,
        }
    }

    /// 检查订单是否可以与另一个订单匹配
    pub fn can_match(&self, other: &Order) -> bool {
        // 必须是不同的方向
        if self.side == other.side {
            return false;
        }

        // 必须是同一个交易对
        if self.symbol != other.symbol {
            return false;
        }

        // 检查价格匹配
        match (self.side, other.side) {
            (OrderSide::Buy, OrderSide::Sell) => {
                // 买单价格 >= 卖单价格
                if let (Some(buy_price), Some(sell_price)) = (self.price, other.price) {
                    buy_price >= sell_price
                } else {
                    // 市价单总是可以匹配
                    true
                }
            }
            (OrderSide::Sell, OrderSide::Buy) => {
                // 卖单价格 <= 买单价格
                if let (Some(sell_price), Some(buy_price)) = (self.price, other.price) {
                    sell_price <= buy_price
                } else {
                    // 市价单总是可以匹配
                    true
                }
            }
            _ => false,
        }
    }

    /// 计算匹配价格（价格优先原则）
    pub fn match_price(&self, other: &Order) -> f64 {
        match (self.side, other.side) {
            (OrderSide::Buy, OrderSide::Sell) => {
                // 买单与卖单匹配，使用先进入市场的价格
                if self.timestamp <= other.timestamp {
                    other.price.unwrap_or(0.0) // 卖单价格
                } else {
                    self.price.unwrap_or(0.0) // 买单价格
                }
            }
            (OrderSide::Sell, OrderSide::Buy) => {
                // 卖单与买单匹配，使用先进入市场的价格
                if self.timestamp <= other.timestamp {
                    self.price.unwrap_or(0.0) // 卖单价格
                } else {
                    other.price.unwrap_or(0.0) // 买单价格
                }
            }
            _ => 0.0,
        }
    }
}

/// 交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub symbol: Symbol,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub quantity: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub buyer_id: String,
    pub seller_id: String,
}

impl Trade {
    pub fn new(
        symbol: Symbol,
        buy_order: &Order,
        sell_order: &Order,
        quantity: f64,
        price: f64,
    ) -> Self {
        let id = Uuid::new_v4();
        let timestamp = Utc::now();

        let (buy_order_id, sell_order_id, buyer_id, seller_id) =
            match (buy_order.side, sell_order.side) {
                (OrderSide::Buy, OrderSide::Sell) => (
                    buy_order.id,
                    sell_order.id,
                    buy_order.user_id.clone(),
                    sell_order.user_id.clone(),
                ),
                (OrderSide::Sell, OrderSide::Buy) => (
                    sell_order.id,
                    buy_order.id,
                    sell_order.user_id.clone(),
                    buy_order.user_id.clone(),
                ),
                _ => panic!("Invalid order sides for trade"),
            };

        Self {
            id,
            symbol,
            buy_order_id,
            sell_order_id,
            quantity,
            price,
            timestamp,
            buyer_id,
            seller_id,
        }
    }
}

/// 订单簿条目
#[derive(Debug, Clone)]
pub struct OrderBookEntry {
    pub order: Order,
    pub priority: u64, // 时间优先级，越小越优先
}

impl OrderBookEntry {
    pub fn new(order: Order, priority: u64) -> Self {
        Self { order, priority }
    }
}

/// 价格级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub total_quantity: f64,
    pub order_count: usize,
}

/// 订单簿深度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookDepth {
    pub symbol: Symbol,
    pub bids: Vec<PriceLevel>, // 买盘，价格从高到低
    pub asks: Vec<PriceLevel>, // 卖盘，价格从低到高
    pub timestamp: DateTime<Utc>,
}

/// 市场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: Symbol,
    pub last_price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub timestamp: DateTime<Utc>,
}

/// API 请求和响应类型
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub order_id: Uuid,
    pub status: OrderStatus,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: Uuid,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrderBookRequest {
    pub symbol: Symbol,
    pub depth: Option<usize>,
}

/// WebSocket 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "trade")]
    Trade(Trade),
    #[serde(rename = "orderbook")]
    OrderBook(OrderBookDepth),
    #[serde(rename = "market_data")]
    MarketData(MarketData),
    #[serde(rename = "order_update")]
    OrderUpdate(Order),
    #[serde(rename = "error")]
    Error { message: String },
}

/// 撮合引擎统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub total_orders: u64,
    pub total_trades: u64,
    pub total_volume: f64,
    pub active_orders: u64,
    pub uptime_seconds: u64,
}
