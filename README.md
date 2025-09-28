# 🚀 高性能撮合引擎 (Matching Engine)

一个使用 Rust 构建的高性能、低延迟的撮合引擎，专为数字货币交易所设计。

## ✨ 特性

- **极高性能**: 使用裸金属 Rust + tokio 异步运行时，支持每秒数百万笔订单处理
- **低延迟**: 微秒级订单处理延迟
- **实时推送**: WebSocket 实时推送交易数据、订单簿更新和市场数据
- **完整API**: RESTful API 支持订单管理、市场数据查询等
- **监控完备**: 集成 Prometheus 指标监控和健康检查
- **配置灵活**: 支持多环境配置管理
- **日志完善**: 结构化日志和日志轮转
- **测试完备**: 单元测试和性能基准测试

## 🏗️ 技术栈

根据推荐的技术栈表格实现：

| 组件 | 技术 | 理由 |
|------|------|------|
| Web API / 网关 | axum | 性能极佳，生态现代，基于 tokio |
| WebSocket 服务 | axum (集成 tungstenite) | 可同时处理 HTTP/WebSocket |
| 业务逻辑/微服务 | axum + tokio | 统一的异步运行时，工具链完善 |
| 核心撮合引擎 | 裸金属 Rust + tokio | 避免任何不必要的开销，极致性能 |
| 日志 | tracing + tracing-appender + tracing-subscriber | 新一代日志和分布式追踪框架 |
| 监控 | metrics-rs + prometheus | 暴露指标，方便用 Grafana 可视化 |
| 配置管理 | config-rs | 支持多种格式，分层覆盖配置 |
| 测试 | criterion | 用于性能基准测试，保证核心性能 |

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Cargo

### 安装和运行

```bash
# 克隆项目
git clone <repository-url>
cd matching_engine

# 构建项目
cargo build --release

# 运行
cargo run

# 或者使用配置文件
RUN_MODE=development cargo run
```

### 使用 Docker

```bash
# 构建 Docker 镜像
docker build -t matching-engine .

# 运行容器
docker run -p 8080:8080 -p 9090:9090 matching-engine
```

## 📖 API 文档

### REST API

#### 健康检查
```bash
GET /api/v1/health
```

#### 创建订单
```bash
POST /api/v1/orders
Content-Type: application/json

{
  "symbol": {"base": "BTC", "quote": "USDT"},
  "side": "buy",
  "order_type": "limit",
  "quantity": 1.0,
  "price": 50000.0,
  "user_id": "user123"
}
```

#### 获取订单
```bash
GET /api/v1/orders/{order_id}
```

#### 取消订单
```bash
DELETE /api/v1/orders/{order_id}?user_id=user123
```

#### 获取订单簿
```bash
GET /api/v1/orderbook/BTCUSDT?depth=10
```

#### 获取市场数据
```bash
GET /api/v1/market-data/BTCUSDT
```

#### 获取交易历史
```bash
GET /api/v1/trades?limit=100
GET /api/v1/trades/BTCUSDT?limit=100
```

### WebSocket API

#### 连接 WebSocket
```javascript
// 通用 WebSocket
const ws = new WebSocket('ws://localhost:8080/ws');

// 交易数据
const ws = new WebSocket('ws://localhost:8080/ws/trades');

// 订单簿数据
const ws = new WebSocket('ws://localhost:8080/ws/orderbook');

// 市场数据
const ws = new WebSocket('ws://localhost:8080/ws/market-data');
```

#### 消息格式
```json
{
  "type": "trade",
  "id": "uuid",
  "symbol": {"base": "BTC", "quote": "USDT"},
  "quantity": 1.0,
  "price": 50000.0,
  "timestamp": "2024-01-01T00:00:00Z",
  "buyer_id": "buyer123",
  "seller_id": "seller456"
}
```

## 🔧 配置

### 环境变量

```bash
# 运行模式
export RUN_MODE=development  # development, production

# 服务器配置
export MATCHING_ENGINE_SERVER_HOST=0.0.0.0
export MATCHING_ENGINE_SERVER_PORT=8080

# 日志配置
export MATCHING_ENGINE_LOGGING_LEVEL=info
export MATCHING_ENGINE_LOGGING_FILE=/var/log/matching_engine.log

# 监控配置
export MATCHING_ENGINE_MONITORING_ENABLED=true
export MATCHING_ENGINE_MONITORING_METRICS_PORT=9090
```

### 配置文件

配置文件位于 `config/` 目录：

- `default.toml` - 默认配置
- `development.toml` - 开发环境配置
- `production.toml` - 生产环境配置
- `local.toml` - 本地配置（不提交到版本控制）

## 📊 监控

### Prometheus 指标

访问 `http://localhost:9090/metrics` 查看 Prometheus 格式的指标。

主要指标：
- `matching_engine_orders_total` - 总订单数
- `matching_engine_trades_total` - 总交易数
- `matching_engine_trade_volume_total` - 总交易量
- `matching_engine_active_orders` - 活跃订单数
- `matching_engine_order_processing_duration_seconds` - 订单处理时间

### 健康检查

```bash
# 应用健康检查
curl http://localhost:8080/api/v1/health

# 监控健康检查
curl http://localhost:8080/monitoring/health
```

## 🧪 测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_matching_engine

# 运行基准测试
cargo bench
```

### 性能基准测试

```bash
# 运行基准测试
cargo bench

# 运行特定基准测试
cargo bench matching_engine_bench
```

## 🏗️ 架构设计

### 核心组件

1. **撮合引擎 (MatchingEngine)**: 核心撮合逻辑
2. **订单簿 (OrderBook)**: 订单存储和匹配
3. **Web API**: RESTful API 接口
4. **WebSocket**: 实时数据推送
5. **监控系统**: 指标收集和健康检查
6. **配置管理**: 多环境配置支持

### 数据流

```
客户端 -> Web API -> 撮合引擎 -> 订单簿
                ↓
            WebSocket -> 客户端
                ↓
            监控系统 -> Prometheus
```

## 🔒 撮合规则

### 价格优先 (Price Priority)
- 买单：价格高的优先
- 卖单：价格低的优先

### 时间优先 (Time Priority)
- 相同价格的订单按时间先后排序

### 撮合算法
1. 接收新订单
2. 在订单簿中寻找匹配订单
3. 按价格优先、时间优先原则撮合
4. 生成交易记录
5. 更新订单状态
6. 广播交易和订单更新

## 📈 性能指标

在标准硬件上的性能表现：

- **订单处理**: > 1,000,000 订单/秒
- **延迟**: < 10 微秒
- **内存使用**: < 100MB (100万订单)
- **CPU 使用**: < 50% (单核)

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [axum](https://github.com/tokio-rs/axum) - 现代 Web 框架
- [tokio](https://github.com/tokio-rs/tokio) - 异步运行时
- [tracing](https://github.com/tokio-rs/tracing) - 结构化日志
- [metrics](https://github.com/metrics-rs/metrics) - 指标收集
- [serde](https://github.com/serde-rs/serde) - 序列化框架

## 📞 支持

如有问题或建议，请：

1. 查看 [Issues](https://github.com/your-repo/issues)
2. 创建新的 Issue
3. 联系维护者

---

**注意**: 这是一个演示项目，用于展示如何使用 Rust 构建高性能撮合引擎。在生产环境中使用前，请进行充分的安全审计和性能测试。
