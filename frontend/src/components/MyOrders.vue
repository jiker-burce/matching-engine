<template>
  <div class="my-orders">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <el-icon><Document /></el-icon>
          <span>我的订单</span>
          <div class="header-controls">
            <el-select v-model="statusFilter" size="small" style="width: 120px">
              <el-option label="全部" value="all" />
              <el-option label="待成交" value="pending" />
              <el-option label="已成交" value="filled" />
              <el-option label="已取消" value="cancelled" />
            </el-select>
            <el-button size="small" @click="refreshOrders">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </div>
      </template>
      
      <div class="orders-container">
        <!-- 表头 -->
        <div class="orders-header">
          <div class="header-cell">时间</div>
          <div class="header-cell">交易对</div>
          <div class="header-cell">类型</div>
          <div class="header-cell">方向</div>
          <div class="header-cell">价格</div>
          <div class="header-cell">数量</div>
          <div class="header-cell">已成交</div>
          <div class="header-cell">状态</div>
          <div class="header-cell">操作</div>
        </div>
        
        <!-- 订单列表 -->
        <div class="orders-list">
          <div 
            v-for="order in filteredOrders" 
            :key="order.id"
            class="order-row"
            :class="order.side"
          >
            <div class="order-cell time-cell">
              <span class="time">{{ formatTime(order.created_at) }}</span>
            </div>
            <div class="order-cell symbol-cell">
              <span class="symbol">{{ order.symbol }}</span>
            </div>
            <div class="order-cell type-cell">
              <el-tag 
                :type="order.type === 'limit' ? 'primary' : 'warning'"
                size="small"
                effect="plain"
              >
                {{ order.type === 'limit' ? '限价' : '市价' }}
              </el-tag>
            </div>
            <div class="order-cell side-cell">
              <el-tag 
                :type="order.side === 'buy' ? 'success' : 'danger'"
                size="small"
                effect="plain"
              >
                {{ order.side === 'buy' ? '买入' : '卖出' }}
              </el-tag>
            </div>
            <div class="order-cell price-cell">
              <span class="price">{{ formatPrice(order.price) }}</span>
            </div>
            <div class="order-cell quantity-cell">
              <span class="quantity">{{ formatQuantity(order.quantity) }}</span>
            </div>
            <div class="order-cell filled-cell">
              <span class="filled">{{ formatQuantity(order.filled_quantity || 0) }}</span>
            </div>
            <div class="order-cell status-cell">
              <el-tag 
                :type="getStatusType(order.status)"
                size="small"
                effect="plain"
              >
                {{ getStatusText(order.status) }}
              </el-tag>
            </div>
            <div class="order-cell action-cell">
              <el-button 
                v-if="order.status === 'pending'"
                size="small"
                type="danger"
                @click="cancelOrder(order.id)"
                :loading="cancellingOrder === order.id"
              >
                取消
              </el-button>
              <span v-else class="no-action">--</span>
            </div>
          </div>
        </div>
        
        <!-- 空状态 -->
        <div v-if="filteredOrders.length === 0" class="empty-state">
          <el-empty description="暂无订单" />
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useTradingStore } from '../stores/trading'
import { ElMessage, ElMessageBox } from 'element-plus'
import dayjs from 'dayjs'
import axios from 'axios'

const tradingStore = useTradingStore()

const statusFilter = ref('all')
const cancellingOrder = ref(null)

// 计算过滤后的订单
const filteredOrders = computed(() => {
  const orders = tradingStore.myOrders || []
  
  if (statusFilter.value === 'all') {
    return orders
  }
  
  return orders.filter(order => {
    switch (statusFilter.value) {
      case 'pending':
        return order.status === 'pending'
      case 'filled':
        return order.status === 'filled'
      case 'cancelled':
        return order.status === 'cancelled'
      default:
        return true
    }
  })
})

// 格式化时间
const formatTime = (timestamp) => {
  if (!timestamp) return '--'
  return dayjs(timestamp).format('MM-DD HH:mm:ss')
}

// 格式化价格
const formatPrice = (price) => {
  if (!price) return '--'
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

// 获取状态类型
const getStatusType = (status) => {
  switch (status) {
    case 'pending':
      return 'warning'
    case 'filled':
      return 'success'
    case 'cancelled':
      return 'info'
    case 'rejected':
      return 'danger'
    default:
      return 'info'
  }
}

// 获取状态文本
const getStatusText = (status) => {
  switch (status) {
    case 'pending':
      return '待成交'
    case 'filled':
      return '已成交'
    case 'cancelled':
      return '已取消'
    case 'rejected':
      return '已拒绝'
    default:
      return '未知'
  }
}

// 取消订单
const cancelOrder = async (orderId) => {
  try {
    await ElMessageBox.confirm(
      '确定要取消这个订单吗？',
      '确认取消',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    cancellingOrder.value = orderId
    
    const result = await tradingStore.cancelOrder(orderId)
    
    if (result.success) {
      ElMessage.success('订单已取消')
    } else {
      ElMessage.error(result.error || '取消订单失败')
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('取消订单失败: ' + error.message)
    }
  } finally {
    cancellingOrder.value = null
  }
}

// 刷新订单
const refreshOrders = async () => {
  console.log('🔄 刷新订单数据...')
  try {
    // 调用真实的订单API
    const response = await axios.get('/api/orders/user123', {
      timeout: 5000
    })
    
    if (response.data && response.data.orders) {
      tradingStore.myOrders = response.data.orders
      console.log('✅ 订单数据已刷新:', response.data.orders.length, '个订单')
    } else {
      tradingStore.myOrders = []
      console.log('✅ 订单数据已刷新: 无订单')
    }
  } catch (error) {
    console.error('❌ 刷新订单失败:', error)
    // 显示错误提示
    ElMessage.error('服务器繁忙，请稍后再试')
    throw error
  }
}

// 注意：已移除模拟数据生成函数，只使用真实API数据

onMounted(() => {
  refreshOrders()
})
</script>

<style scoped>
.my-orders {
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

.orders-container {
  height: 600px;
  display: flex;
  flex-direction: column;
}

.orders-header {
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
.header-cell:nth-child(2) { flex: 0.8; }
.header-cell:nth-child(3) { flex: 0.6; }
.header-cell:nth-child(4) { flex: 0.6; }
.header-cell:nth-child(5) { flex: 1; }
.header-cell:nth-child(6) { flex: 1; }
.header-cell:nth-child(7) { flex: 1; }
.header-cell:nth-child(8) { flex: 0.8; }
.header-cell:nth-child(9) { flex: 0.8; }

.orders-list {
  flex: 1;
  overflow-y: auto;
}

.order-row {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #f0f0f0;
  font-size: 12px;
  transition: background-color 0.2s;
}

.order-row:hover {
  background-color: #f5f7fa;
}

.order-row.buy {
  background-color: rgba(103, 194, 58, 0.02);
}

.order-row.sell {
  background-color: rgba(245, 108, 108, 0.02);
}

.order-cell {
  display: flex;
  align-items: center;
}

.order-cell:nth-child(1) { flex: 1.2; justify-content: flex-start; }
.order-cell:nth-child(2) { flex: 0.8; justify-content: center; }
.order-cell:nth-child(3) { flex: 0.6; justify-content: center; }
.order-cell:nth-child(4) { flex: 0.6; justify-content: center; }
.order-cell:nth-child(5) { flex: 1; justify-content: center; }
.order-cell:nth-child(6) { flex: 1; justify-content: center; }
.order-cell:nth-child(7) { flex: 1; justify-content: center; }
.order-cell:nth-child(8) { flex: 0.8; justify-content: center; }
.order-cell:nth-child(9) { flex: 0.8; justify-content: center; }

.time {
  color: #909399;
}

.symbol {
  color: #303133;
  font-weight: bold;
}

.price {
  font-weight: bold;
  color: #303133;
}

.quantity {
  color: #606266;
}

.filled {
  color: #606266;
}

.no-action {
  color: #c0c4cc;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
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
