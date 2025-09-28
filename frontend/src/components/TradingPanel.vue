<template>
  <div class="trading-panel">
    <!-- 市场信息 -->
    <el-card class="market-info" shadow="never">
      <template #header>
        <div class="card-header">
          <el-icon><TrendCharts /></el-icon>
          <span>市场信息</span>
        </div>
      </template>
      
      <div class="market-data">
        <div class="price-info">
          <div class="current-price">
            <span class="price">¥{{ formatPrice(tradingStore.currentPrice) }}</span>
            <el-tag 
              :type="tradingStore.priceChange >= 0 ? 'success' : 'danger'"
              size="small"
            >
              {{ tradingStore.priceChange >= 0 ? '+' : '' }}{{ tradingStore.priceChangePercent.toFixed(2) }}%
            </el-tag>
          </div>
          <div class="price-details">
            <div class="detail-item">
              <span class="label">24h最高:</span>
              <span class="value">¥{{ formatPrice(tradingStore.marketData.high_24h) }}</span>
            </div>
            <div class="detail-item">
              <span class="label">24h最低:</span>
              <span class="value">¥{{ formatPrice(tradingStore.marketData.low_24h) }}</span>
            </div>
            <div class="detail-item">
              <span class="label">24h成交量:</span>
              <span class="value">{{ formatVolume(tradingStore.volume24h) }}</span>
            </div>
          </div>
          </div>
        </div>
    </el-card>

    <!-- 交易表单 -->
    <el-card class="trading-form" shadow="never">
      <template #header>
        <div class="card-header">
          <el-icon><Money /></el-icon>
          <span>交易</span>
        </div>
      </template>
      
      <el-form :model="orderForm" label-width="80px" size="small">
        <!-- 交易对选择 -->
        <el-form-item label="交易对">
          <el-select v-model="orderForm.symbol" placeholder="选择交易对">
            <el-option label="BTC/USDT" value="BTC-USDT" />
            <el-option label="ETH/USDT" value="ETH-USDT" />
            <el-option label="BNB/USDT" value="BNB-USDT" />
          </el-select>
        </el-form-item>
        
        <!-- 买卖切换 -->
        <el-form-item>
          <el-radio-group v-model="orderForm.side" class="side-selector">
            <el-radio-button label="buy" class="buy-button">
              <el-icon><ArrowUp /></el-icon>
              买入
            </el-radio-button>
            <el-radio-button label="sell" class="sell-button">
              <el-icon><ArrowDown /></el-icon>
              卖出
            </el-radio-button>
          </el-radio-group>
        </el-form-item>
        
        <!-- 订单类型 -->
        <el-form-item label="类型">
          <el-select v-model="orderForm.type" placeholder="订单类型">
            <el-option label="限价单" value="limit" />
            <el-option label="市价单" value="market" />
          </el-select>
        </el-form-item>
        
        <!-- 价格输入 -->
        <el-form-item label="价格" v-if="orderForm.type === 'limit'">
          <el-input 
            v-model="orderForm.price" 
            placeholder="输入价格"
            type="number"
            :min="0"
            :step="0.01"
          >
            <template #append>USDT</template>
          </el-input>
        </el-form-item>
        
        <!-- 数量输入 -->
        <el-form-item label="数量">
          <el-input 
            v-model="orderForm.quantity" 
            placeholder="输入数量"
            type="number"
            :min="0"
            :step="0.001"
          >
            <template #append>{{ orderForm.symbol.split('-')[0] }}</template>
          </el-input>
        </el-form-item>
        
        <!-- 总金额显示 -->
        <el-form-item v-if="orderForm.type === 'limit' && orderForm.price && orderForm.quantity">
          <div class="total-amount">
            <span class="label">总金额:</span>
            <span class="value">{{ formatPrice(parseFloat(orderForm.price) * parseFloat(orderForm.quantity)) }} USDT</span>
          </div>
        </el-form-item>
        
        <!-- 提交按钮 -->
        <el-form-item>
          <el-button 
            type="primary" 
            :class="orderForm.side"
            @click="submitOrder"
            :loading="submitting"
            block
          >
            {{ orderForm.side === 'buy' ? '买入' : '卖出' }} {{ orderForm.symbol.split('-')[0] }}
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 快速操作 -->
    <el-card class="quick-actions" shadow="never">
      <template #header>
        <div class="card-header">
          <el-icon><Lightning /></el-icon>
          <span>快速操作</span>
        </div>
      </template>
      
      <div class="quick-buttons">
        <el-button 
          v-for="percent in [25, 50, 75, 100]" 
          :key="percent"
          size="small"
          @click="setQuantityPercent(percent)"
        >
          {{ percent }}%
        </el-button>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref, reactive, computed } from 'vue'
import { useTradingStore } from '../stores/trading'
import { ElMessage } from 'element-plus'

const tradingStore = useTradingStore()

// 订单表单
const orderForm = reactive({
  symbol: 'BTC-USDT',
  side: 'buy',
  type: 'limit',
  price: '',
  quantity: ''
})

const submitting = ref(false)

// 提交订单
const submitOrder = async () => {
  if (!orderForm.quantity || (orderForm.type === 'limit' && !orderForm.price)) {
    ElMessage.warning('请填写完整的订单信息')
    return
  }
  
  submitting.value = true
  
  try {
    const orderData = {
      symbol: orderForm.symbol,
      side: orderForm.side,
      type: orderForm.type,
      quantity: parseFloat(orderForm.quantity),
      price: orderForm.type === 'limit' ? parseFloat(orderForm.price) : null
    }
    
    const result = await tradingStore.submitOrder(orderData)
    
    if (result.success) {
      ElMessage.success('订单提交成功')
      // 重置表单
      orderForm.price = ''
      orderForm.quantity = ''
    } else {
      ElMessage.error(result.error || '订单提交失败')
    }
  } catch (error) {
    ElMessage.error('订单提交失败: ' + error.message)
  } finally {
    submitting.value = false
  }
}

// 设置数量百分比
const setQuantityPercent = (percent) => {
  // 这里应该根据用户余额计算，暂时使用固定值
  const maxQuantity = 1.0 // 假设最大数量为1
  orderForm.quantity = (maxQuantity * percent / 100).toFixed(3)
}

// 格式化价格
const formatPrice = (price) => {
  if (!price) return '0.00'
  return parseFloat(price).toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })
}

// 格式化成交量
const formatVolume = (volume) => {
  if (!volume) return '0'
  if (volume >= 1e9) {
    return (volume / 1e9).toFixed(2) + 'B'
  } else if (volume >= 1e6) {
    return (volume / 1e6).toFixed(2) + 'M'
  } else if (volume >= 1e3) {
    return (volume / 1e3).toFixed(2) + 'K'
  }
  return volume.toFixed(2)
}
</script>

<style scoped>
.trading-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-header {
  display: flex;
  align-items: center;
  font-weight: bold;
}

.card-header .el-icon {
  margin-right: 8px;
}

.market-data {
  padding: 8px 0;
}

.price-info {
  margin-bottom: 16px;
}

.current-price {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.price {
  font-size: 24px;
  font-weight: bold;
  color: #303133;
}

.price-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}

.label {
  color: #909399;
}

.value {
  color: #303133;
  font-weight: 500;
}

.side-selector {
  width: 100%;
}

.side-selector .el-radio-button {
  flex: 1;
}

.buy-button {
  color: #67c23a;
}

.sell-button {
  color: #f56c6c;
}

.total-amount {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  background: #f5f7fa;
  border-radius: 4px;
  font-size: 14px;
}

.total-amount .label {
  color: #909399;
}

.total-amount .value {
  color: #303133;
  font-weight: bold;
}

.quick-buttons {
  display: flex;
  gap: 8px;
}

.quick-buttons .el-button {
  flex: 1;
}

:deep(.el-button.buy) {
  background: #67c23a;
  border-color: #67c23a;
}

:deep(.el-button.sell) {
  background: #f56c6c;
  border-color: #f56c6c;
}
</style>
