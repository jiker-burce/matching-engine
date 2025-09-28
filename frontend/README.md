# 撮合引擎前端交易界面

这是一个基于Vue3的现代化交易界面，集成了实时WebSocket数据、免费数据源和完整的交易功能。

## 功能特性

### 🚀 核心功能
- **实时交易界面** - 现代化的交易面板，支持限价单和市价单
- **K线图表** - 集成ECharts的专业K线图，支持多种时间周期
- **订单簿** - 实时显示买卖盘深度，支持价格时间优先级
- **交易历史** - 完整的交易记录和统计
- **我的订单** - 订单管理和状态跟踪

### 📊 数据源集成
- **CoinGecko API** - 免费的市场数据源
- **WebSocket实时数据** - 与后端撮合引擎实时同步
- **技术指标** - RSI、MACD、移动平均线等

### 🎨 界面特性
- **响应式设计** - 适配不同屏幕尺寸
- **暗色主题** - 专业的交易界面风格
- **实时更新** - WebSocket驱动的实时数据
- **交互友好** - 直观的操作体验

## 技术栈

- **Vue 3** - 现代化的前端框架
- **Element Plus** - UI组件库
- **Pinia** - 状态管理
- **ECharts** - 图表库
- **Axios** - HTTP客户端
- **Day.js** - 时间处理
- **Vite** - 构建工具

## 快速开始

### 1. 安装依赖
```bash
cd frontend
npm install
```

### 2. 启动开发服务器
```bash
npm run dev
```

### 3. 访问应用
打开浏览器访问 `http://localhost:3300`

## 项目结构

```
frontend/
├── src/
│   ├── components/          # 组件
│   │   ├── TradingPanel.vue    # 交易面板
│   │   ├── TradingView.vue     # 交易视图（K线图）
│   │   ├── OrderBook.vue       # 订单簿
│   │   ├── TradeHistory.vue   # 交易历史
│   │   └── MyOrders.vue        # 我的订单
│   ├── stores/              # 状态管理
│   │   └── trading.js           # 交易状态
│   ├── views/               # 页面
│   ├── router/              # 路由
│   ├── App.vue              # 根组件
│   └── main.js              # 入口文件
├── package.json
├── vite.config.js
└── README.md
```

## 主要组件

### TradingPanel.vue
- 市场信息显示
- 交易表单（买入/卖出）
- 快速操作按钮
- 订单类型选择

### TradingView.vue
- K线图表显示
- 技术指标计算
- 时间周期切换
- 图表交互

### OrderBook.vue
- 实时订单簿显示
- 买卖盘深度
- 价差计算
- 数量强度显示

### TradeHistory.vue
- 交易历史列表
- 实时交易更新
- 分页加载
- 交易对筛选

### MyOrders.vue
- 订单管理
- 状态筛选
- 订单取消
- 订单详情

## 数据源配置

### CoinGecko API
```javascript
// 免费的市场数据
const COINGECKO_API = 'https://api.coingecko.com/api/v3'

// 获取价格数据
const response = await axios.get(`${COINGECKO_API}/simple/price`, {
  params: {
    ids: 'bitcoin,ethereum,binancecoin',
    vs_currencies: 'usdt',
    include_24hr_change: true,
    include_24hr_vol: true
  }
})
```

### WebSocket连接
```javascript
// 实时数据连接
const ws = new WebSocket('ws://localhost:8888/ws')

ws.onmessage = (event) => {
  const data = JSON.parse(event.data)
  // 处理实时数据
}
```

## 开发说明

### 环境要求
- Node.js 16+
- npm 或 yarn

### 开发命令
```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build

# 预览生产版本
npm run preview
```

### 代理配置
前端通过Vite代理连接到后端：
- API请求: `/api/*` → `http://localhost:8888`
- WebSocket: `/ws` → `ws://localhost:8888`

## 部署说明

### 构建生产版本
```bash
npm run build
```

### 部署到服务器
将 `dist` 目录部署到Web服务器即可。

## 注意事项

1. **数据源限制** - CoinGecko免费API有请求频率限制
2. **WebSocket连接** - 需要后端服务器运行在8888端口
3. **浏览器兼容性** - 支持现代浏览器（Chrome、Firefox、Safari、Edge）
4. **移动端适配** - 界面已做响应式处理，支持移动端访问

## 故障排除

### 常见问题

1. **WebSocket连接失败**
   - 检查后端服务器是否运行
   - 确认端口8888是否可访问

2. **数据加载失败**
   - 检查网络连接
   - 确认CoinGecko API是否可访问

3. **图表不显示**
   - 检查ECharts是否正确加载
   - 确认数据格式是否正确

## 贡献指南

欢迎提交Issue和Pull Request来改进这个项目！

## 许可证

MIT License
