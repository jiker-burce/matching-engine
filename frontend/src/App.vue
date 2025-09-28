<template>
  <div id="app">
    <el-container class="app-container">
      <!-- 顶部导航栏 -->
      <el-header class="app-header">
        <div class="header-content">
          <div class="logo">
            <el-icon><TrendCharts /></el-icon>
            <span>撮合引擎交易平台</span>
          </div>
          <div class="header-info">
            <el-tag type="success" v-if="connectionStatus === 'connected'">
              <el-icon><Connection /></el-icon>
              已连接
            </el-tag>
            <el-tag type="danger" v-else>
              <el-icon><Close /></el-icon>
              未连接
            </el-tag>
            <span class="server-time">{{ currentTime }}</span>
          </div>
        </div>
      </el-header>

      <!-- 主要内容区域 -->
      <el-container>
        <!-- 侧边栏 -->
        <el-aside width="300px" class="sidebar">
          <TradingPanel />
        </el-aside>

        <!-- 主内容区 -->
        <el-main class="main-content">
          <el-tabs v-model="activeTab" type="border-card">
            <el-tab-pane label="交易" name="trading">
              <TradingView />
            </el-tab-pane>
            <el-tab-pane label="订单簿" name="orderbook">
              <OrderBook />
            </el-tab-pane>
            <el-tab-pane label="交易历史" name="trades">
              <TradeHistory />
            </el-tab-pane>
            <el-tab-pane label="我的订单" name="orders">
              <MyOrders />
            </el-tab-pane>
          </el-tabs>
        </el-main>
      </el-container>
    </el-container>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { useTradingStore } from './stores/trading'
import TradingPanel from './components/TradingPanel.vue'
import TradingView from './components/TradingView.vue'
import OrderBook from './components/OrderBook.vue'
import TradeHistory from './components/TradeHistory.vue'
import MyOrders from './components/MyOrders.vue'

const tradingStore = useTradingStore()

const activeTab = ref('trading')
const connectionStatus = ref('disconnected')
const currentTime = ref('')

// 更新时间
const updateTime = () => {
  currentTime.value = new Date().toLocaleTimeString('zh-CN')
}

onMounted(() => {
  updateTime()
  setInterval(updateTime, 1000)
  
  // 初始化交易数据
  tradingStore.initialize()
  
  // 监听连接状态
  tradingStore.$subscribe((mutation, state) => {
    connectionStatus.value = state.websocketConnected ? 'connected' : 'disconnected'
  })
})

onUnmounted(() => {
  tradingStore.disconnect()
})
</script>

<style scoped>
.app-container {
  height: 100vh;
}

.app-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  display: flex;
  align-items: center;
  padding: 0 20px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.logo {
  display: flex;
  align-items: center;
  font-size: 20px;
  font-weight: bold;
}

.logo .el-icon {
  margin-right: 8px;
  font-size: 24px;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.server-time {
  font-family: 'Courier New', monospace;
  font-size: 14px;
}

.sidebar {
  background: #f5f7fa;
  border-right: 1px solid #e4e7ed;
  padding: 16px;
}

.main-content {
  padding: 0;
  background: #fafafa;
}

:deep(.el-tabs__content) {
  padding: 0;
}
</style>
