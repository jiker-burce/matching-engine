use crate::orderbook::{SafeOrderBook};
use crate::types::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::broadcast;
use tracing::info;
use uuid::Uuid;
use chrono::Utc;

/// 撮合引擎核心实现
#[derive(Debug)]
pub struct MatchingEngine {
    /// 每个交易对的订单簿
    orderbooks: Arc<RwLock<HashMap<Symbol, SafeOrderBook>>>,
    /// 所有订单的存储
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    /// 交易历史
    trades: Arc<RwLock<Vec<Trade>>>,
    /// 市场数据
    market_data: Arc<RwLock<HashMap<Symbol, MarketData>>>,
    /// 统计信息
    stats: Arc<RwLock<EngineStats>>,
    /// 启动时间
    start_time: Instant,
    /// 交易广播通道
    trade_sender: broadcast::Sender<Trade>,
    /// 订单更新广播通道
    order_sender: broadcast::Sender<Order>,
    /// 市场数据广播通道
    market_data_sender: broadcast::Sender<MarketData>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        let (trade_sender, _) = broadcast::channel(10000);
        let (order_sender, _) = broadcast::channel(10000);
        let (market_data_sender, _) = broadcast::channel(1000);

        Self {
            orderbooks: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            trades: Arc::new(RwLock::new(Vec::new())),
            market_data: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(EngineStats {
                total_orders: 0,
                total_trades: 0,
                total_volume: 0.0,
                active_orders: 0,
                uptime_seconds: 0,
            })),
            start_time: Instant::now(),
            trade_sender,
            order_sender,
            market_data_sender,
        }
    }

    /// 提交订单进行撮合
    pub async fn submit_order(&self, mut order: Order) -> Result<Vec<Trade>, String> {
        let order_id = order.id;
        let symbol = order.symbol.clone();

        info!("Submitting order {} for {}", order_id, symbol.to_string());

        // 验证订单
        self.validate_order(&order)?;

        // 获取或创建订单簿
        let orderbook = self.get_or_create_orderbook(&symbol);

        // 存储订单
        {
            let mut orders = self.orders.write().unwrap();
            orders.insert(order_id, order.clone());
        }

        // 更新统计信息
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_orders += 1;
            stats.active_orders += 1;
        }

        // 尝试撮合
        let trades = self.match_order(&orderbook, &mut order).await?;

        // 如果订单没有完全成交，添加到订单簿
        if order.remaining_quantity > 0.0 {
            orderbook.add_order(order.clone())?;
            info!("Order {} partially filled, added to orderbook", order_id);
        } else {
            order.status = OrderStatus::Filled;
            info!("Order {} completely filled", order_id);
        }

        // 更新订单状态
        {
            let mut orders = self.orders.write().unwrap();
            orders.insert(order_id, order.clone());
        }

        // 广播订单更新
        let _ = self.order_sender.send(order);

        // 更新市场数据
        self.update_market_data(&symbol).await;

        // 广播市场数据
        if let Some(market_data) = self.get_market_data(&symbol) {
            let _ = self.market_data_sender.send(market_data);
        }

        Ok(trades)
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_id: Uuid, user_id: String) -> Result<Order, String> {
        info!("Cancelling order {} for user {}", order_id, user_id);

        // 获取订单
        let order = {
            let orders = self.orders.read().unwrap();
            orders
                .get(&order_id)
                .cloned()
                .ok_or_else(|| "Order not found".to_string())?
        };

        // 验证用户权限
        if order.user_id != user_id {
            return Err("Unauthorized to cancel this order".to_string());
        }

        // 检查订单状态
        if order.status == OrderStatus::Filled {
            return Err("Cannot cancel filled order".to_string());
        }

        if order.status == OrderStatus::Cancelled {
            return Err("Order already cancelled".to_string());
        }

        // 从订单簿中移除
        let orderbook = self
            .get_orderbook(&order.symbol)
            .ok_or_else(|| "Orderbook not found".to_string())?;

        let mut cancelled_order = orderbook.remove_order(order_id)?;
        cancelled_order.status = OrderStatus::Cancelled;

        // 更新订单存储
        {
            let mut orders = self.orders.write().unwrap();
            orders.insert(order_id, cancelled_order.clone());
        }

        // 更新统计信息
        {
            let mut stats = self.stats.write().unwrap();
            stats.active_orders = stats.active_orders.saturating_sub(1);
        }

        // 广播订单更新
        let _ = self.order_sender.send(cancelled_order.clone());

        info!("Order {} cancelled successfully", order_id);
        Ok(cancelled_order)
    }

    /// 获取订单信息
    pub fn get_order(&self, order_id: Uuid) -> Option<Order> {
        self.orders.read().unwrap().get(&order_id).cloned()
    }

    /// 获取用户的所有订单
    pub fn get_user_orders(&self, user_id: &str) -> Vec<Order> {
        self.orders
            .read()
            .unwrap()
            .values()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect()
    }

    /// 获取订单簿深度
    pub fn get_orderbook_depth(
        &self,
        symbol: &Symbol,
        depth: Option<usize>,
    ) -> Option<OrderBookDepth> {
        self.get_orderbook(symbol)
            .map(|orderbook| orderbook.get_depth(depth))
    }

    /// 获取市场数据
    pub fn get_market_data(&self, symbol: &Symbol) -> Option<MarketData> {
        self.market_data.read().unwrap().get(symbol).cloned()
    }

    /// 获取所有市场数据
    pub fn get_all_market_data(&self) -> HashMap<Symbol, MarketData> {
        self.market_data.read().unwrap().clone()
    }

    /// 获取引擎统计信息
    pub fn get_stats(&self) -> EngineStats {
        let mut stats = self.stats.read().unwrap().clone();
        stats.uptime_seconds = self.start_time.elapsed().as_secs();
        stats
    }

    /// 获取交易历史
    pub fn get_trades(&self, symbol: Option<&Symbol>, limit: Option<usize>) -> Vec<Trade> {
        let trades = self.trades.read().unwrap();
        let mut filtered_trades: Vec<Trade> = trades
            .iter()
            .filter(|trade| {
                if let Some(sym) = symbol {
                    trade.symbol == *sym
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // 按时间倒序排列（最新的在前）
        filtered_trades.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            filtered_trades.truncate(limit);
        }

        filtered_trades
    }

    /// 获取交易广播接收器
    pub fn subscribe_trades(&self) -> broadcast::Receiver<Trade> {
        self.trade_sender.subscribe()
    }

    /// 获取订单更新广播接收器
    pub fn subscribe_orders(&self) -> broadcast::Receiver<Order> {
        self.order_sender.subscribe()
    }

    /// 获取市场数据广播接收器
    pub fn subscribe_market_data(&self) -> broadcast::Receiver<MarketData> {
        self.market_data_sender.subscribe()
    }

    /// 验证订单
    fn validate_order(&self, order: &Order) -> Result<(), String> {
        if order.quantity <= 0.0 {
            return Err("Order quantity must be positive".to_string());
        }

        if order.order_type == OrderType::Limit {
            if let Some(price) = order.price {
                if price <= 0.0 {
                    return Err("Limit order price must be positive".to_string());
                }
            } else {
                return Err("Limit order must have a price".to_string());
            }
        }

        if order.user_id.is_empty() {
            return Err("User ID cannot be empty".to_string());
        }

        Ok(())
    }

    /// 获取或创建订单簿
    fn get_or_create_orderbook(&self, symbol: &Symbol) -> SafeOrderBook {
        let mut orderbooks = self.orderbooks.write().unwrap();
        if !orderbooks.contains_key(symbol) {
            orderbooks.insert(symbol.clone(), SafeOrderBook::new(symbol.clone()));
        }
        orderbooks.get(symbol).unwrap().clone()
    }

    /// 获取订单簿
    fn get_orderbook(&self, symbol: &Symbol) -> Option<SafeOrderBook> {
        self.orderbooks.read().unwrap().get(symbol).cloned()
    }

    /// 撮合订单
    async fn match_order(
        &self,
        orderbook: &SafeOrderBook,
        incoming_order: &mut Order,
    ) -> Result<Vec<Trade>, String> {
        let mut trades = Vec::new();
        let mut remaining_quantity = incoming_order.remaining_quantity;

        // 获取匹配的订单
        let matching_orders = orderbook.get_matching_orders(incoming_order);

        for matching_entry in matching_orders {
            if remaining_quantity <= 0.0 {
                break;
            }

            let matching_order = &matching_entry.order;

            // 检查是否可以匹配
            if !incoming_order.can_match(matching_order) {
                continue;
            }

            // 计算匹配数量
            let match_quantity = remaining_quantity.min(matching_order.remaining_quantity);

            // 计算匹配价格
            let match_price = incoming_order.match_price(matching_order);

            // 创建交易
            let trade = Trade::new(
                incoming_order.symbol.clone(),
                incoming_order,
                matching_order,
                match_quantity,
                match_price,
            );

            // 更新订单数量
            remaining_quantity -= match_quantity;
            incoming_order.filled_quantity += match_quantity;
            incoming_order.remaining_quantity = remaining_quantity;

            // 更新匹配订单
            let new_matching_quantity = matching_order.remaining_quantity - match_quantity;
            orderbook.update_order(matching_order.id, new_matching_quantity)?;

            // 如果匹配订单完全成交，从订单簿中移除
            if new_matching_quantity <= 0.0 {
                let mut filled_order = orderbook.remove_order(matching_order.id)?;
                filled_order.status = OrderStatus::Filled;
                filled_order.filled_quantity = filled_order.quantity;
                filled_order.remaining_quantity = 0.0;

                // 更新订单存储
                {
                    let mut orders = self.orders.write().unwrap();
                    orders.insert(filled_order.id, filled_order.clone());
                }

                // 广播订单更新
                let _ = self.order_sender.send(filled_order);

                // 更新统计信息
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.active_orders = stats.active_orders.saturating_sub(1);
                }
            }

            // 存储交易
            {
                let mut trades_store = self.trades.write().unwrap();
                trades_store.push(trade.clone());
            }

            // 更新统计信息
            {
                let mut stats = self.stats.write().unwrap();
                stats.total_trades += 1;
                stats.total_volume += trade.quantity * trade.price;
            }

            // 广播交易
            let _ = self.trade_sender.send(trade.clone());
            let trade_id = trade.id;
            trades.push(trade);

            info!(
                "Trade executed: {} {} at {} for {}",
                match_quantity,
                incoming_order.symbol.to_string(),
                match_price,
                trade_id
            );
        }

        Ok(trades)
    }

    /// 更新市场数据
    async fn update_market_data(&self, symbol: &Symbol) {
        let orderbook = match self.get_orderbook(symbol) {
            Some(ob) => ob,
            None => return,
        };

        let best_bid = orderbook.best_bid();
        let best_ask = orderbook.best_ask();
        let spread = orderbook.spread();

        // 获取最近的交易来计算24小时数据
        let recent_trades = self.get_trades(Some(symbol), Some(1000));

        let mut volume_24h = 0.0;
        let mut high_24h: f64 = 0.0;
        let mut low_24h: f64 = f64::MAX;
        let mut last_price = 0.0;

        for trade in &recent_trades {
            volume_24h += trade.quantity * trade.price;
            high_24h = high_24h.max(trade.price);
            low_24h = low_24h.min(trade.price);
            last_price = trade.price;
        }

        if low_24h == f64::MAX {
            low_24h = 0.0;
        }

        // 计算24小时价格变化
        let price_change_24h = if recent_trades.len() > 1 {
            let first_price = recent_trades.last().unwrap().price;
            ((last_price - first_price) / first_price) * 100.0
        } else {
            0.0
        };

        let market_data = MarketData {
            symbol: symbol.clone(),
            last_price,
            volume_24h,
            price_change_24h,
            high_24h,
            low_24h,
            timestamp: Utc::now(),
        };

        {
            let mut market_data_store = self.market_data.write().unwrap();
            market_data_store.insert(symbol.clone(), market_data);
        }
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_matching_engine_basic_matching() {
        let engine = MatchingEngine::new();
        let symbol = Symbol::new("BTC", "USDT");

        // 提交卖单
        let sell_order = Order::new(
            symbol.clone(),
            OrderSide::Sell,
            OrderType::Limit,
            1.0,
            Some(50000.0),
            "seller".to_string(),
        );

        let trades = engine.submit_order(sell_order).await.unwrap();
        assert_eq!(trades.len(), 0); // 没有匹配的买单

        // 提交买单
        let buy_order = Order::new(
            symbol.clone(),
            OrderSide::Buy,
            OrderType::Limit,
            1.0,
            Some(50000.0),
            "buyer".to_string(),
        );

        let trades = engine.submit_order(buy_order).await.unwrap();
        assert_eq!(trades.len(), 1); // 应该有一个交易
        assert_eq!(trades[0].quantity, 1.0);
        assert_eq!(trades[0].price, 50000.0);
    }

    #[tokio::test]
    async fn test_matching_engine_partial_fill() {
        let engine = MatchingEngine::new();
        let symbol = Symbol::new("BTC", "USDT");

        // 提交大卖单
        let sell_order = Order::new(
            symbol.clone(),
            OrderSide::Sell,
            OrderType::Limit,
            2.0,
            Some(50000.0),
            "seller".to_string(),
        );

        engine.submit_order(sell_order).await.unwrap();

        // 提交小买单
        let buy_order = Order::new(
            symbol.clone(),
            OrderSide::Buy,
            OrderType::Limit,
            1.0,
            Some(50000.0),
            "buyer".to_string(),
        );

        let trades = engine.submit_order(buy_order).await.unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 1.0);

        // 检查卖单是否部分成交
        let orderbook_depth = engine.get_orderbook_depth(&symbol, None).unwrap();
        assert_eq!(orderbook_depth.asks.len(), 1);
        assert_eq!(orderbook_depth.asks[0].total_quantity, 1.0);
    }
}
