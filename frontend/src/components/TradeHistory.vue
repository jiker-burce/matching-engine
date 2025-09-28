<template>
  <div class="trade-history">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <el-icon><Clock /></el-icon>
          <span>交易历史</span>
          <div class="header-controls">
            <el-select v-model="selectedSymbol" size="small" style="width: 120px">
              <el-option label="BTC/USDT" value="BTC-USDT" />
              <el-option label="ETH/USDT" value="ETH-USDT" />
              <el-option label="BNB/USDT" value="BNB-USDT" />
            </el-select>
            <el-button size="small" @click="refreshTrades">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </div>
      </template>
      
      <div class="trades-container">
        <!-- 表头 -->
        <div class="trades-header">
          <div class="header-cell">时间</div>
          <div class="header-cell">价格</div>
          <div class="header-cell">数量</div>
          <div class="header-cell">方向</div>
          <div class="header-cell">总额</div>
        </div>
        
        <!-- 交易列表 -->
        <div class="trades-list">
          <div 
            v-for="trade in displayTrades" 
            :key="trade.id"
            class="trade-row"
            :class="trade.side"
          >
            <div class="trade-cell time-cell">
              <span class="time">{{ formatTime(trade.timestamp) }}</span>
            </div>
            <div class="trade-cell price-cell">
              <span class="price">{{ formatPrice(trade.price) }}</span>
            </div>
            <div class="trade-cell quantity-cell">
              <span class="quantity">{{ formatQuantity(trade.quantity) }}</span>
            </div>
            <div class="trade-cell side-cell">
              <el-tag 
                :type="trade.side === 'buy' ? 'success' : 'danger'"
                size="small"
                effect="plain"
              >
                {{ trade.side === 'buy' ? '买入' : '卖出' }}
              </el-tag>
            </div>
            <div class="trade-cell total-cell">
              <span class="total">{{ formatPrice(trade.price * trade.quantity) }}</span>
            </div>
          </div>
        </div>
        
        <!-- 加载更多 -->
        <div class="load-more" v-if="hasMore">
          <el-button 
            size="small" 
            @click="loadMoreTrades"
            :loading="loading"
          >
            加载更多
          </el-button>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { useTradingStore } from '../stores/trading'
import dayjs from 'dayjs'

const tradingStore = useTradingStore()

const selectedSymbol = ref('BTC-USDT')
const loading = ref(false)
const hasMore = ref(true)
const currentPage = ref(1)
const pageSize = ref(50)

// 计算显示的交易数据
const displayTrades = computed(() => {
  return tradingStore.trades || []
})

// 格式化时间
const formatTime = (timestamp) => {
  if (!timestamp) return '--'
  return dayjs(timestamp).format('HH:mm:ss')
}

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

// 刷新交易历史
const refreshTrades = async () => {
  loading.value = true
  try {
    await tradingStore.loadTradeHistory()
  } finally {
    loading.value = false
  }
}

// 加载更多交易
const loadMoreTrades = async () => {
  loading.value = true
  try {
    currentPage.value++
    // 这里应该调用API加载更多数据
    // 暂时使用模拟数据
    const mockTrades = generateMockTrades(currentPage.value * pageSize.value)
    tradingStore.trades.push(...mockTrades)
    
    if (mockTrades.length < pageSize.value) {
      hasMore.value = false
    }
  } finally {
    loading.value = false
  }
}

// 生成模拟交易数据
const generateMockTrades = (count) => {
  const trades = []
  const basePrice = tradingStore.currentPrice || 45000
  
  for (let i = 0; i < Math.min(count, 20); i++) {
    const price = basePrice + (Math.random() - 0.5) * 1000
    const quantity = Math.random() * 2 + 0.01
    const side = Math.random() > 0.5 ? 'buy' : 'sell'
    
    trades.push({
      id: `trade_${Date.now()}_${i}`,
      price: price,
      quantity: quantity,
      side: side,
      timestamp: dayjs().subtract(i, 'minute').toISOString()
    })
  }
  
  return trades
}

// 监听交易对变化
watch(selectedSymbol, () => {
  refreshTrades()
})

onMounted(() => {
  refreshTrades()
})
</script>

<style scoped>
.trade-history {
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

.trades-container {
  height: 600px;
  display: flex;
  flex-direction: column;
}

.trades-header {
  display: flex;
  background: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  font-size: 12px;
  font-weight: bold;
  color: #909399;
}

.header-cell {
  padding: 8px 12px;
  text-align: center;
}

.header-cell:nth-child(1) { flex: 1.2; text-align: left; }
.header-cell:nth-child(2) { flex: 1; }
.header-cell:nth-child(3) { flex: 1; }
.header-cell:nth-child(4) { flex: 0.8; }
.header-cell:nth-child(5) { flex: 1; text-align: right; }

.trades-list {
  flex: 1;
  overflow-y: auto;
}

.trade-row {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #f0f0f0;
  font-size: 12px;
  transition: background-color 0.2s;
}

.trade-row:hover {
  background-color: #f5f7fa;
}

.trade-row.buy {
  background-color: rgba(103, 194, 58, 0.05);
}

.trade-row.sell {
  background-color: rgba(245, 108, 108, 0.05);
}

.trade-cell {
  display: flex;
  align-items: center;
}

.trade-cell:nth-child(1) { flex: 1.2; justify-content: flex-start; }
.trade-cell:nth-child(2) { flex: 1; justify-content: center; }
.trade-cell:nth-child(3) { flex: 1; justify-content: center; }
.trade-cell:nth-child(4) { flex: 0.8; justify-content: center; }
.trade-cell:nth-child(5) { flex: 1; justify-content: flex-end; }

.time {
  color: #909399;
}

.price {
  font-weight: bold;
  color: #303133;
}

.quantity {
  color: #606266;
}

.total {
  font-weight: bold;
  color: #303133;
}

.load-more {
  padding: 16px;
  text-align: center;
  border-top: 1px solid #e4e7ed;
}

/* 滚动条样式 */
.trades-list::-webkit-scrollbar {
  width: 4px;
}

.trades-list::-webkit-scrollbar-track {
  background: #f1f1f1;
}

.trades-list::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 2px;
}

.trades-list::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}
</style>
