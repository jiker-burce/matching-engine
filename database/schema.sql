-- 交易系统数据库设计
-- 使用 PostgreSQL + TimescaleDB

-- 创建扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "timescaledb";

-- 用户表
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 账户表
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    currency VARCHAR(10) NOT NULL,
    balance DECIMAL(20, 8) DEFAULT 0.0,
    frozen_balance DECIMAL(20, 8) DEFAULT 0.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, currency)
);

-- 交易对表
CREATE TABLE trading_pairs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    symbol VARCHAR(20) UNIQUE NOT NULL,
    base_currency VARCHAR(10) NOT NULL,
    quote_currency VARCHAR(10) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    min_order_size DECIMAL(20, 8) DEFAULT 0.001,
    max_order_size DECIMAL(20, 8) DEFAULT 1000000.0,
    price_precision INTEGER DEFAULT 2,
    quantity_precision INTEGER DEFAULT 8,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 订单表
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    trading_pair_id UUID NOT NULL REFERENCES trading_pairs(id),
    order_type VARCHAR(10) NOT NULL CHECK (order_type IN ('limit', 'market')),
    side VARCHAR(4) NOT NULL CHECK (side IN ('buy', 'sell')),
    price DECIMAL(20, 8),
    quantity DECIMAL(20, 8) NOT NULL,
    filled_quantity DECIMAL(20, 8) DEFAULT 0.0,
    remaining_quantity DECIMAL(20, 8) GENERATED ALWAYS AS (quantity - filled_quantity) STORED,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'partially_filled', 'filled', 'cancelled', 'rejected')),
    time_in_force VARCHAR(10) DEFAULT 'GTC' CHECK (time_in_force IN ('GTC', 'IOC', 'FOK')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    filled_at TIMESTAMP WITH TIME ZONE,
    cancelled_at TIMESTAMP WITH TIME ZONE
);

-- 创建订单表的时间序列分区
SELECT create_hypertable('orders', 'created_at');

-- 交易表
CREATE TABLE trades (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trading_pair_id UUID NOT NULL REFERENCES trading_pairs(id),
    buy_order_id UUID REFERENCES orders(id),
    sell_order_id UUID REFERENCES orders(id),
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    total_amount DECIMAL(20, 8) GENERATED ALWAYS AS (price * quantity) STORED,
    taker_side VARCHAR(4) NOT NULL CHECK (taker_side IN ('buy', 'sell')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 创建交易表的时间序列分区
SELECT create_hypertable('trades', 'created_at');

-- 订单簿快照表（用于历史数据）
CREATE TABLE orderbook_snapshots (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trading_pair_id UUID NOT NULL REFERENCES trading_pairs(id),
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 创建订单簿快照表的时间序列分区
SELECT create_hypertable('orderbook_snapshots', 'created_at');

-- 市场数据表
CREATE TABLE market_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trading_pair_id UUID NOT NULL REFERENCES trading_pairs(id),
    price DECIMAL(20, 8) NOT NULL,
    volume_24h DECIMAL(20, 8) DEFAULT 0.0,
    high_24h DECIMAL(20, 8),
    low_24h DECIMAL(20, 8),
    price_change_24h DECIMAL(20, 8) DEFAULT 0.0,
    price_change_percentage_24h DECIMAL(10, 4) DEFAULT 0.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 创建市场数据表的时间序列分区
SELECT create_hypertable('market_data', 'created_at');

-- 索引优化
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_trading_pair ON orders(trading_pair_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at DESC);
CREATE INDEX idx_orders_price_side ON orders(price, side) WHERE status = 'pending';

CREATE INDEX idx_trades_trading_pair ON trades(trading_pair_id);
CREATE INDEX idx_trades_created_at ON trades(created_at DESC);
CREATE INDEX idx_trades_buy_order ON trades(buy_order_id);
CREATE INDEX idx_trades_sell_order ON trades(sell_order_id);

CREATE INDEX idx_market_data_trading_pair ON market_data(trading_pair_id);
CREATE INDEX idx_market_data_created_at ON market_data(created_at DESC);

-- 创建视图：活跃订单簿
CREATE VIEW active_orderbook AS
SELECT 
    tp.symbol,
    o.side,
    o.price,
    SUM(o.remaining_quantity) as total_quantity,
    COUNT(*) as order_count
FROM orders o
JOIN trading_pairs tp ON o.trading_pair_id = tp.id
WHERE o.status = 'pending' AND o.remaining_quantity > 0
GROUP BY tp.symbol, o.side, o.price
ORDER BY tp.symbol, o.side, o.price;

-- 创建视图：24小时交易统计
CREATE VIEW daily_trading_stats AS
SELECT 
    tp.symbol,
    DATE(t.created_at) as trade_date,
    COUNT(*) as trade_count,
    SUM(t.quantity) as total_volume,
    AVG(t.price) as avg_price,
    MIN(t.price) as min_price,
    MAX(t.price) as max_price,
    SUM(t.total_amount) as total_amount
FROM trades t
JOIN trading_pairs tp ON t.trading_pair_id = tp.id
WHERE t.created_at >= NOW() - INTERVAL '24 hours'
GROUP BY tp.symbol, DATE(t.created_at)
ORDER BY tp.symbol, trade_date DESC;
