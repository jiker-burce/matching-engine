<template>
  <div class="data-source-test">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>🔍 数据源测试</span>
          <el-button type="primary" @click="testDataSources">测试所有数据源</el-button>
        </div>
      </template>
      
      <div class="test-results">
        <h3>📊 数据源状态</h3>
        <div v-for="(result, index) in testResults" :key="index" class="result-item">
          <div class="result-header">
            <span class="source-name">{{ result.source }}</span>
            <el-tag :type="result.success ? 'success' : 'danger'">
              {{ result.success ? '✅ 成功' : '❌ 失败' }}
            </el-tag>
          </div>
          <div v-if="result.success" class="result-data">
            <p><strong>价格:</strong> ${{ result.data.price }}</p>
            <p><strong>24h变化:</strong> {{ result.data.price_change_24h }}%</p>
            <p><strong>成交量:</strong> {{ formatVolume(result.data.total_volume) }}</p>
          </div>
          <div v-else class="result-error">
            <p><strong>错误:</strong> {{ result.error }}</p>
          </div>
        </div>
      </div>
      
      <div class="instructions">
        <h4>📝 使用说明</h4>
        <p>点击"测试所有数据源"按钮，然后查看浏览器控制台，你会看到详细的日志输出：</p>
        <ul>
          <li>🔄 数据加载开始</li>
          <li>🔍 尝试各个数据源</li>
          <li>🔗 API调用详情</li>
          <li>✅ 成功响应或 ❌ 失败信息</li>
          <li>📊 模拟数据标识</li>
          <li>🌐 真实API数据标识</li>
        </ul>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useTradingStore } from '../stores/trading.js'

const tradingStore = useTradingStore()
const testResults = ref([])

const testDataSources = async () => {
  console.log('🧪 开始测试所有数据源...')
  testResults.value = []
  
  // 测试各个数据源
  const strategies = [
    { name: 'Backend API', test: () => tradingStore.marketDataService.strategies[0].fetchData() },
    { name: 'Binance API', test: () => tradingStore.marketDataService.strategies[1].fetchData() },
    { name: 'CoinGecko API', test: () => tradingStore.marketDataService.strategies[2].fetchData() },
    { name: 'Default Data', test: () => tradingStore.marketDataService.strategies[3].fetchData() }
  ]
  
  for (const strategy of strategies) {
    try {
      console.log(`🧪 测试 ${strategy.name}...`)
      const data = await strategy.test()
      testResults.value.push({
        source: strategy.name,
        success: true,
        data: data
      })
      console.log(`✅ ${strategy.name} 测试成功`)
    } catch (error) {
      testResults.value.push({
        source: strategy.name,
        success: false,
        error: error.message
      })
      console.log(`❌ ${strategy.name} 测试失败:`, error.message)
    }
  }
  
  console.log('🎉 数据源测试完成!')
}

const formatVolume = (volume) => {
  if (volume >= 1e9) return `${(volume / 1e9).toFixed(2)}B`
  if (volume >= 1e6) return `${(volume / 1e6).toFixed(2)}M`
  if (volume >= 1e3) return `${(volume / 1e3).toFixed(2)}K`
  return volume.toString()
}
</script>

<style scoped>
.data-source-test {
  margin: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.test-results {
  margin: 20px 0;
}

.result-item {
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  padding: 15px;
  margin: 10px 0;
  background: #fafafa;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.source-name {
  font-weight: bold;
  font-size: 16px;
}

.result-data p {
  margin: 5px 0;
  color: #606266;
}

.result-error p {
  margin: 5px 0;
  color: #f56c6c;
}

.instructions {
  margin-top: 20px;
  padding: 15px;
  background: #f0f9ff;
  border-radius: 4px;
}

.instructions h4 {
  margin-top: 0;
  color: #1890ff;
}

.instructions ul {
  margin: 10px 0;
  padding-left: 20px;
}

.instructions li {
  margin: 5px 0;
  color: #606266;
}
</style>
