<template>
  <div class="orderbook">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <el-icon><List /></el-icon>
          <span>订单簿</span>
          <div class="header-controls">
            <el-select v-model="depth" size="small" style="width: 100px">
              <el-option label="10档" value="10" />
              <el-option label="20档" value="20" />
              <el-option label="50档" value="50" />
            </el-select>
            <el-button size="small" @click="refreshOrderBook">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </div>
      </template>
      
      <div class="orderbook-container">
        <!-- 卖盘 -->
        <div class="asks-section">
          <div class="section-header">
            <span class="section-title">卖盘 (Asks)</span>
            <span class="section-info">价格 | 数量 | 累计</span>
          </div>
          <div class="orders-list asks-list">
            <div 
              v-for="(ask, index) in displayAsks" 
              :key="`ask-${index}`"
              class="order-row ask-row"
              :style="{ background: `linear-gradient(to right, rgba(245, 108, 108, ${ask.intensity}) 0%, transparent 100%)` }"
            >
              <span class="price ask-price">{{ formatPrice(ask.price) }}</span>
              <span class="quantity">{{ formatQuantity(ask.quantity) }}</span>
              <span class="total">{{ formatQuantity(ask.total) }}</span>
            </div>
          </div>
        </div>
        
        <!-- 中间分隔线 -->
        <div class="spread-section">
          <div class="spread-info">
            <div class="spread-value">
              <span class="label">价差:</span>
              <span class="value">{{ formatPrice(spread) }}</span>
            </div>
            <div class="spread-percent">
              <span class="label">价差%:</span>
              <span class="value">{{ spreadPercent.toFixed(4) }}%</span>
            </div>
          </div>
        </div>
        
        <!-- 买盘 -->
        <div class="bids-section">
          <div class="section-header">
            <span class="section-title">买盘 (Bids)</span>
            <span class="section-info">价格 | 数量 | 累计</span>
          </div>
          <div class="orders-list bids-list">
            <div 
              v-for="(bid, index) in displayBids" 
              :key="`bid-${index}`"
              class="order-row bid-row"
              :style="{ background: `linear-gradient(to right, rgba(103, 194, 58, ${bid.intensity}) 0%, transparent 100%)` }"
            >
              <span class="price bid-price">{{ formatPrice(bid.price) }}</span>
              <span class="quantity">{{ formatQuantity(bid.quantity) }}</span>
              <span class="total">{{ formatQuantity(bid.total) }}</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { useTradingStore } from '../stores/trading'

const tradingStore = useTradingStore()

const depth = ref('20')

// 计算显示的买卖盘数据
const displayAsks = computed(() => {
  const asks = tradingStore.orderBook.asks || []
  const maxDepth = parseInt(depth.value)
  const limitedAsks = asks.slice(0, maxDepth)
  
  // 计算累计数量和强度
  let cumulativeTotal = 0
  const maxQuantity = Math.max(...limitedAsks.map(item => item.quantity), 1)
  
  return limitedAsks.map((ask, index) => {
    cumulativeTotal += ask.quantity
    return {
      ...ask,
      total: cumulativeTotal,
      intensity: ask.quantity / maxQuantity
    }
  }).reverse() // 卖盘从高到低显示
})

const displayBids = computed(() => {
  const bids = tradingStore.orderBook.bids || []
  const maxDepth = parseInt(depth.value)
  const limitedBids = bids.slice(0, maxDepth)
  
  // 计算累计数量和强度
  let cumulativeTotal = 0
  const maxQuantity = Math.max(...limitedBids.map(item => item.quantity), 1)
  
  return limitedBids.map((bid, index) => {
    cumulativeTotal += bid.quantity
    return {
      ...bid,
      total: cumulativeTotal,
      intensity: bid.quantity / maxQuantity
    }
  })
})

// 计算价差
const spread = computed(() => {
  const asks = tradingStore.orderBook.asks || []
  const bids = tradingStore.orderBook.bids || []
  
  if (asks.length === 0 || bids.length === 0) return 0
  
  const bestAsk = asks[0]?.price || 0
  const bestBid = bids[0]?.price || 0
  
  return bestAsk - bestBid
})

// 计算价差百分比
const spreadPercent = computed(() => {
  const asks = tradingStore.orderBook.asks || []
  const bids = tradingStore.orderBook.bids || []
  
  if (asks.length === 0 || bids.length === 0) return 0
  
  const bestAsk = asks[0]?.price || 0
  const bestBid = bids[0]?.price || 0
  
  if (bestBid === 0) return 0
  
  return ((bestAsk - bestBid) / bestBid) * 100
})

// 格式化价格
const formatPrice = (price) => {
  if (!price) return '0.00'
  return parseFloat(price).toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })
}

// 格式化数量
const formatQuantity = (quantity) => {
  if (!quantity) return '0.000'
  return parseFloat(quantity).toLocaleString('zh-CN', {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3
  })
}

// 刷新订单簿
const refreshOrderBook = () => {
  tradingStore.loadOrderBook()
}

// 监听深度变化
watch(depth, () => {
  // 深度变化时重新计算显示数据
})

onMounted(() => {
  // 组件挂载时加载订单簿数据
  tradingStore.loadOrderBook()
})
</script>

<style scoped>
.orderbook {
  height: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header .el-icon {
  margin-right: 8px;
}

.header-controls {
  display: flex;
  gap: 8px;
  align-items: center;
}

.orderbook-container {
  display: flex;
  flex-direction: column;
  height: 600px;
}

.asks-section {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.bids-section {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  font-size: 12px;
  font-weight: bold;
}

.section-title {
  color: #303133;
}

.section-info {
  color: #909399;
}

.orders-list {
  flex: 1;
  overflow-y: auto;
}

.order-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 12px;
  font-size: 12px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.order-row:hover {
  background-color: rgba(64, 158, 255, 0.1) !important;
}

.ask-row {
  color: #f56c6c;
}

.bid-row {
  color: #67c23a;
}

.price {
  flex: 1;
  text-align: left;
  font-weight: bold;
}

.quantity {
  flex: 1;
  text-align: center;
}

.total {
  flex: 1;
  text-align: right;
  color: #909399;
}

.spread-section {
  padding: 12px;
  background: #fafafa;
  border-top: 1px solid #e4e7ed;
  border-bottom: 1px solid #e4e7ed;
}

.spread-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.spread-value,
.spread-percent {
  display: flex;
  align-items: center;
  gap: 4px;
}

.label {
  color: #909399;
}

.value {
  color: #303133;
  font-weight: bold;
}

/* 滚动条样式 */
.orders-list::-webkit-scrollbar {
  width: 4px;
}

.orders-list::-webkit-scrollbar-track {
  background: #f1f1f1;
}

.orders-list::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 2px;
}

.orders-list::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}
</style>
