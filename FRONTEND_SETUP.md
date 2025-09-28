# 🚀 撮合引擎前端交易界面 - 完整设置指南

## 📋 项目概述

这是一个完整的撮合引擎系统，包含：
- **后端**: Rust + Tokio + Axum 高性能撮合引擎
- **前端**: Vue3 + Element Plus 现代化交易界面
- **数据源**: CoinGecko免费API + WebSocket实时数据

## 🛠️ 系统要求

- **Rust** 1.70+ (后端)
- **Node.js** 16+ (前端)
- **npm** 或 **yarn** (包管理)

## 🚀 快速启动

### 方法一：使用启动脚本（推荐）

```bash
# 在项目根目录运行
./start.sh
```

这个脚本会自动：
1. 启动后端服务器 (端口 8888)
2. 安装前端依赖
3. 启动前端开发服务器 (端口 3000)

### 方法二：手动启动

#### 1. 启动后端服务器
```bash
# 在项目根目录
cargo run
```

#### 2. 启动前端服务器
```bash
# 新开一个终端，进入前端目录
cd frontend
npm install
npm run dev
```

## 🌐 访问应用

- **前端交易界面**: http://localhost:3300
- **后端API**: http://localhost:8888
- **健康检查**: http://localhost:8888/health
- **引擎统计**: http://localhost:8888/stats

## 📊 功能演示

### 1. 交易面板
- 实时市场数据（价格、涨跌幅、成交量）
- 买入/卖出订单提交
- 限价单/市价单支持
- 快速数量选择（25%, 50%, 75%, 100%）

### 2. K线图表
- 专业K线图显示
- 多种时间周期（1m, 5m, 15m, 1h, 4h, 1d）
- 技术指标（RSI, MACD, MA20, MA50）
- 实时数据更新

### 3. 订单簿
- 实时买卖盘深度
- 价格时间优先级显示
- 价差计算
- 数量强度可视化

### 4. 交易历史
- 实时交易记录
- 交易对筛选
- 分页加载
- 时间格式化

### 5. 我的订单
- 订单状态管理
- 订单取消功能
- 状态筛选
- 订单详情显示

## 🔧 技术特性

### 前端技术栈
- **Vue 3** - 组合式API
- **Element Plus** - UI组件库
- **Pinia** - 状态管理
- **ECharts** - 图表库
- **Axios** - HTTP客户端
- **Vite** - 构建工具

### 数据源集成
- **CoinGecko API** - 免费市场数据
- **WebSocket** - 实时数据推送
- **模拟数据** - 开发测试支持

### 实时功能
- WebSocket连接状态显示
- 实时价格更新
- 实时订单簿更新
- 实时交易记录

## 📁 项目结构

```
matching_engine/
├── src/                    # Rust后端源码
│   ├── main.rs
│   ├── matching_engine.rs
│   ├── orderbook.rs
│   ├── types.rs
│   └── ...
├── frontend/               # Vue3前端
│   ├── src/
│   │   ├── components/     # 组件
│   │   ├── stores/         # 状态管理
│   │   ├── views/          # 页面
│   │   └── ...
│   ├── package.json
│   └── vite.config.js
├── start.sh               # 启动脚本
└── README.md
```

## 🎯 使用指南

### 1. 查看市场数据
- 访问 http://localhost:3300
- 查看实时价格和涨跌幅
- 观察24小时最高/最低价

### 2. 提交交易订单
- 选择交易对（BTC/USDT）
- 选择买入/卖出
- 选择订单类型（限价/市价）
- 输入价格和数量
- 点击提交按钮

### 3. 查看订单簿
- 切换到"订单簿"标签
- 查看实时买卖盘深度
- 观察价差和数量分布

### 4. 查看交易历史
- 切换到"交易历史"标签
- 查看实时交易记录
- 筛选不同交易对

### 5. 管理我的订单
- 切换到"我的订单"标签
- 查看订单状态
- 取消待成交订单

## 🔍 故障排除

### 常见问题

1. **端口冲突**
   ```bash
   # 检查端口占用
   lsof -i :8888
   lsof -i :3300
   
   # 杀死占用进程
   kill -9 <PID>
   ```

2. **依赖安装失败**
   ```bash
   # 清理缓存
   npm cache clean --force
   
   # 重新安装
   rm -rf node_modules package-lock.json
   npm install
   ```

3. **WebSocket连接失败**
   - 检查后端服务器是否运行
   - 确认防火墙设置
   - 检查网络连接

4. **数据加载失败**
   - 检查CoinGecko API访问
   - 确认网络代理设置
   - 查看浏览器控制台错误

### 调试模式

```bash
# 后端调试
RUST_LOG=debug cargo run

# 前端调试
npm run dev -- --debug
```

## 📈 性能优化

### 后端优化
- 使用Rust的高性能特性
- Tokio异步运行时
- 内存池管理
- 零拷贝数据传输

### 前端优化
- Vue3组合式API
- 虚拟滚动（大数据列表）
- 图表懒加载
- WebSocket连接池

## 🚀 部署说明

### 开发环境
```bash
# 后端
cargo run

# 前端
npm run dev
```

### 生产环境
```bash
# 后端
cargo build --release
./target/release/matching_engine

# 前端
npm run build
# 部署 dist 目录到Web服务器
```

## 📚 学习资源

- [Rust官方文档](https://doc.rust-lang.org/)
- [Vue3官方文档](https://vuejs.org/)
- [Element Plus文档](https://element-plus.org/)
- [ECharts文档](https://echarts.apache.org/)

## 🤝 贡献指南

欢迎提交Issue和Pull Request！

1. Fork项目
2. 创建功能分支
3. 提交更改
4. 发起Pull Request

## 📄 许可证

MIT License

---

🎉 **恭喜！你现在拥有了一个完整的撮合引擎交易系统！**
