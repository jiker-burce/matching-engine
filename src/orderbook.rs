use crate::types::*;
use chrono::Utc;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use tracing::debug;
use uuid::Uuid;

/// 订单簿实现
/// 使用 BTreeMap 来维护价格优先，时间优先的排序
#[derive(Debug)]
pub struct OrderBook {
    symbol: Symbol,
    // 买盘：价格从高到低排序 (BTreeMap 默认升序，我们使用负数来实现降序)
    bids: BTreeMap<i64, Vec<OrderBookEntry>>,
    // 卖盘：价格从低到高排序
    asks: BTreeMap<i64, Vec<OrderBookEntry>>,
    // 订单ID到价格的映射，用于快速查找和删除
    order_price_map: HashMap<Uuid, (OrderSide, i64)>,
    // 时间优先级计数器
    priority_counter: u64,
}

impl OrderBook {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_price_map: HashMap::new(),
            priority_counter: 0,
        }
    }

    /// 添加订单到订单簿
    pub fn add_order(&mut self, mut order: Order) -> Result<(), String> {
        if order.symbol != self.symbol {
            return Err(format!(
                "Order symbol {} does not match orderbook symbol {}",
                order.symbol.to_string(),
                self.symbol.to_string()
            ));
        }

        if order.remaining_quantity <= 0.0 {
            return Err("Order quantity must be positive".to_string());
        }

        // 设置时间优先级
        let priority = self.priority_counter;
        self.priority_counter += 1;

        let entry = OrderBookEntry::new(order.clone(), priority);

        // 将价格转换为整数以避免浮点数精度问题
        let price_key = self.price_to_key(order.price.unwrap_or(0.0));

        // 根据订单方向添加到相应的订单簿
        match order.side {
            OrderSide::Buy => {
                // 买盘：使用负数价格键来实现降序排序
                let price_key = -price_key;
                self.bids
                    .entry(price_key)
                    .or_insert_with(Vec::new)
                    .push(entry);
                self.order_price_map
                    .insert(order.id, (OrderSide::Buy, price_key));
            }
            OrderSide::Sell => {
                // 卖盘：使用正数价格键来实现升序排序
                self.asks
                    .entry(price_key)
                    .or_insert_with(Vec::new)
                    .push(entry);
                self.order_price_map
                    .insert(order.id, (OrderSide::Sell, price_key));
            }
        }

        debug!(
            "Added order {} to orderbook for {}",
            order.id,
            self.symbol.to_string()
        );
        Ok(())
    }

    /// 从订单簿中移除订单
    pub fn remove_order(&mut self, order_id: Uuid) -> Result<Order, String> {
        let (side, price_key) = self
            .order_price_map
            .remove(&order_id)
            .ok_or_else(|| "Order not found".to_string())?;

        let orderbook = match side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };

        let entries = orderbook
            .get_mut(&price_key)
            .ok_or_else(|| "Price level not found".to_string())?;

        // 找到并移除订单
        let index = entries
            .iter()
            .position(|entry| entry.order.id == order_id)
            .ok_or_else(|| "Order not found in price level".to_string())?;

        let entry = entries.remove(index);

        // 如果价格级别为空，移除整个级别
        if entries.is_empty() {
            orderbook.remove(&price_key);
        }

        debug!(
            "Removed order {} from orderbook for {}",
            order_id,
            self.symbol.to_string()
        );
        Ok(entry.order)
    }

    /// 更新订单
    pub fn update_order(&mut self, order_id: Uuid, new_quantity: f64) -> Result<Order, String> {
        let (side, price_key) = self
            .order_price_map
            .get(&order_id)
            .ok_or_else(|| "Order not found".to_string())?;

        let orderbook = match side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };

        let entries = orderbook
            .get_mut(price_key)
            .ok_or_else(|| "Price level not found".to_string())?;

        let index = entries
            .iter()
            .position(|entry| entry.order.id == order_id)
            .ok_or_else(|| "Order not found in price level".to_string())?;

        let entry = &mut entries[index];
        let old_quantity = entry.order.remaining_quantity;
        entry.order.remaining_quantity = new_quantity;
        entry.order.filled_quantity = entry.order.quantity - new_quantity;

        // 更新订单状态
        if new_quantity <= 0.0 {
            entry.order.status = OrderStatus::Filled;
        } else if entry.order.filled_quantity > 0.0 {
            entry.order.status = OrderStatus::PartiallyFilled;
        }

        debug!(
            "Updated order {} quantity from {} to {}",
            order_id, old_quantity, new_quantity
        );

        Ok(entry.order.clone())
    }

    /// 获取最佳买价
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.keys().next().map(|&key| self.key_to_price(-key))
    }

    /// 获取最佳卖价
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.keys().next().map(|&key| self.key_to_price(key))
    }

    /// 获取买卖价差
    pub fn spread(&self) -> Option<f64> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask - bid),
            _ => None,
        }
    }

    /// 获取订单簿深度
    pub fn get_depth(&self, max_depth: Option<usize>) -> OrderBookDepth {
        let depth = max_depth.unwrap_or(10);

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        // 获取买盘深度（价格从高到低）
        for (&price_key, entries) in self.bids.iter().take(depth) {
            let total_quantity: f64 = entries.iter().map(|e| e.order.remaining_quantity).sum();
            bids.push(PriceLevel {
                price: self.key_to_price(-price_key),
                total_quantity,
                order_count: entries.len(),
            });
        }

        // 获取卖盘深度（价格从低到高）
        for (&price_key, entries) in self.asks.iter().take(depth) {
            let total_quantity: f64 = entries.iter().map(|e| e.order.remaining_quantity).sum();
            asks.push(PriceLevel {
                price: self.key_to_price(price_key),
                total_quantity,
                order_count: entries.len(),
            });
        }

        OrderBookDepth {
            symbol: self.symbol.clone(),
            bids,
            asks,
            timestamp: Utc::now(),
        }
    }

    /// 获取匹配的订单（价格优先，时间优先）
    pub fn get_matching_orders(&self, incoming_order: &Order) -> Vec<OrderBookEntry> {
        let mut matching_orders = Vec::new();

        match incoming_order.side {
            OrderSide::Buy => {
                // 买单匹配卖盘，寻找价格 <= 买单价格的卖单
                if let Some(price) = incoming_order.price {
                    let max_price_key = self.price_to_key(price);

                    for (&price_key, entries) in self.asks.iter() {
                        if price_key > max_price_key {
                            break; // 价格太高，停止搜索
                        }

                        // 按时间优先排序（priority 越小越优先）
                        let mut sorted_entries = entries.clone();
                        sorted_entries.sort_by_key(|e| e.priority);
                        matching_orders.extend(sorted_entries);
                    }
                } else {
                    // 市价买单，匹配所有卖单
                    for (_, entries) in self.asks.iter() {
                        let mut sorted_entries = entries.clone();
                        sorted_entries.sort_by_key(|e| e.priority);
                        matching_orders.extend(sorted_entries);
                    }
                }
            }
            OrderSide::Sell => {
                // 卖单匹配买盘，寻找价格 >= 卖单价格的买单
                if let Some(price) = incoming_order.price {
                    let min_price_key = self.price_to_key(price);

                    for (&price_key, entries) in self.bids.iter() {
                        if -price_key < min_price_key {
                            break; // 价格太低，停止搜索
                        }

                        // 按时间优先排序（priority 越小越优先）
                        let mut sorted_entries = entries.clone();
                        sorted_entries.sort_by_key(|e| e.priority);
                        matching_orders.extend(sorted_entries);
                    }
                } else {
                    // 市价卖单，匹配所有买单
                    for (_, entries) in self.bids.iter() {
                        let mut sorted_entries = entries.clone();
                        sorted_entries.sort_by_key(|e| e.priority);
                        matching_orders.extend(sorted_entries);
                    }
                }
            }
        }

        matching_orders
    }

    /// 获取订单簿统计信息
    pub fn get_stats(&self) -> OrderBookStats {
        let total_bid_orders: usize = self.bids.values().map(|v| v.len()).sum();
        let total_ask_orders: usize = self.asks.values().map(|v| v.len()).sum();
        let total_bid_quantity: f64 = self
            .bids
            .values()
            .flat_map(|v| v.iter())
            .map(|e| e.order.remaining_quantity)
            .sum();
        let total_ask_quantity: f64 = self
            .asks
            .values()
            .flat_map(|v| v.iter())
            .map(|e| e.order.remaining_quantity)
            .sum();

        OrderBookStats {
            symbol: self.symbol.clone(),
            bid_levels: self.bids.len(),
            ask_levels: self.asks.len(),
            total_bid_orders,
            total_ask_orders,
            total_bid_quantity,
            total_ask_quantity,
        }
    }

    /// 将价格转换为整数键（避免浮点数精度问题）
    fn price_to_key(&self, price: f64) -> i64 {
        (price * 1_000_000.0) as i64 // 保留6位小数精度
    }

    /// 将整数键转换回价格
    fn key_to_price(&self, key: i64) -> f64 {
        key as f64 / 1_000_000.0
    }
}

/// 订单簿统计信息
#[derive(Debug, Clone)]
pub struct OrderBookStats {
    pub symbol: Symbol,
    pub bid_levels: usize,
    pub ask_levels: usize,
    pub total_bid_orders: usize,
    pub total_ask_orders: usize,
    pub total_bid_quantity: f64,
    pub total_ask_quantity: f64,
}

/// 线程安全的订单簿包装器
#[derive(Debug, Clone)]
pub struct SafeOrderBook {
    inner: Arc<RwLock<OrderBook>>,
}

impl SafeOrderBook {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            inner: Arc::new(RwLock::new(OrderBook::new(symbol))),
        }
    }

    pub fn add_order(&self, order: Order) -> Result<(), String> {
        self.inner.write().unwrap().add_order(order)
    }

    pub fn remove_order(&self, order_id: Uuid) -> Result<Order, String> {
        self.inner.write().unwrap().remove_order(order_id)
    }

    pub fn update_order(&self, order_id: Uuid, new_quantity: f64) -> Result<Order, String> {
        self.inner
            .write()
            .unwrap()
            .update_order(order_id, new_quantity)
    }

    pub fn best_bid(&self) -> Option<f64> {
        self.inner.read().unwrap().best_bid()
    }

    pub fn best_ask(&self) -> Option<f64> {
        self.inner.read().unwrap().best_ask()
    }

    pub fn spread(&self) -> Option<f64> {
        self.inner.read().unwrap().spread()
    }

    pub fn get_depth(&self, max_depth: Option<usize>) -> OrderBookDepth {
        self.inner.read().unwrap().get_depth(max_depth)
    }

    pub fn get_matching_orders(&self, incoming_order: &Order) -> Vec<OrderBookEntry> {
        self.inner
            .read()
            .unwrap()
            .get_matching_orders(incoming_order)
    }

    pub fn get_stats(&self) -> OrderBookStats {
        self.inner.read().unwrap().get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook_basic_operations() {
        let symbol = Symbol::new("BTC", "USDT");
        let mut orderbook = OrderBook::new(symbol.clone());

        // 添加买单
        let buy_order = Order::new(
            symbol.clone(),
            OrderSide::Buy,
            OrderType::Limit,
            1.0,
            Some(50000.0),
            "user1".to_string(),
        );

        orderbook.add_order(buy_order.clone()).unwrap();
        assert_eq!(orderbook.best_bid(), Some(50000.0));
        assert_eq!(orderbook.best_ask(), None);

        // 添加卖单
        let sell_order = Order::new(
            symbol.clone(),
            OrderSide::Sell,
            OrderType::Limit,
            1.0,
            Some(51000.0),
            "user2".to_string(),
        );

        orderbook.add_order(sell_order.clone()).unwrap();
        assert_eq!(orderbook.best_ask(), Some(51000.0));
        assert_eq!(orderbook.spread(), Some(1000.0));

        // 测试匹配
        let matching_orders = orderbook.get_matching_orders(&buy_order);
        assert_eq!(matching_orders.len(), 1);
        assert_eq!(matching_orders[0].order.id, sell_order.id);
    }

    #[test]
    fn test_price_priority() {
        let symbol = Symbol::new("BTC", "USDT");
        let mut orderbook = OrderBook::new(symbol.clone());

        // 添加多个不同价格的买单
        let order1 = Order::new(
            symbol.clone(),
            OrderSide::Buy,
            OrderType::Limit,
            1.0,
            Some(50000.0),
            "user1".to_string(),
        );
        let order2 = Order::new(
            symbol.clone(),
            OrderSide::Buy,
            OrderType::Limit,
            1.0,
            Some(51000.0),
            "user2".to_string(),
        );
        let order3 = Order::new(
            symbol.clone(),
            OrderSide::Buy,
            OrderType::Limit,
            1.0,
            Some(49000.0),
            "user3".to_string(),
        );

        orderbook.add_order(order1).unwrap();
        orderbook.add_order(order2).unwrap();
        orderbook.add_order(order3).unwrap();

        // 最佳买价应该是51000（最高价格）
        assert_eq!(orderbook.best_bid(), Some(51000.0));
    }
}
