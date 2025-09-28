<template>
  <div class="trading-view">
    <!-- K线图 -->
    <el-card class="chart-card" shadow="never">
      <template #header>
        <div class="chart-header">
          <div class="chart-title">
            <el-icon><TrendCharts /></el-icon>
            <span>K线图</span>
          </div>
          <div class="chart-controls">
            <el-select v-model="timeframe" size="small" style="width: 100px">
              <el-option label="1分钟" value="1m" />
              <el-option label="5分钟" value="5m" />
              <el-option label="15分钟" value="15m" />
              <el-option label="1小时" value="1h" />
              <el-option label="4小时" value="4h" />
              <el-option label="1天" value="1d" />
            </el-select>
            <el-button size="small" @click="refreshChart">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </div>
      </template>
      
      <div class="chart-container">
        <v-chart 
          :option="chartOption" 
          :style="{ height: '400px' }"
          @click="onChartClick"
        />
      </div>
    </el-card>

    <!-- 技术指标 -->
    <el-card class="indicators-card" shadow="never">
      <template #header>
        <div class="card-header">
          <el-icon><DataAnalysis /></el-icon>
          <span>技术指标</span>
        </div>
      </template>
      
      <div class="indicators-grid">
        <div class="indicator-item">
          <div class="indicator-label">RSI (14)</div>
          <div class="indicator-value" :class="getRSIClass(rsiValue)">
            {{ rsiValue.toFixed(2) }}
          </div>
        </div>
        <div class="indicator-item">
          <div class="indicator-label">MACD</div>
          <div class="indicator-value" :class="getMACDClass(macdValue)">
            {{ macdValue.toFixed(4) }}
          </div>
        </div>
        <div class="indicator-item">
          <div class="indicator-label">MA (20)</div>
          <div class="indicator-value">
            {{ formatPrice(ma20Value) }}
          </div>
        </div>
        <div class="indicator-item">
          <div class="indicator-label">MA (50)</div>
          <div class="indicator-value">
            {{ formatPrice(ma50Value) }}
          </div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart, CandlestickChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import { useTradingStore } from '../stores/trading'

// 注册ECharts组件
use([
  CanvasRenderer,
  LineChart,
  CandlestickChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent
])

const tradingStore = useTradingStore()

const timeframe = ref('1h')
const chartData = ref([])
const rsiValue = ref(50)
const macdValue = ref(0)
const ma20Value = ref(0)
const ma50Value = ref(0)

// 图表配置
const chartOption = computed(() => {
  return {
    title: {
      text: 'BTC/USDT',
      left: 'center'
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross'
      }
    },
    legend: {
      data: ['K线', 'MA20', 'MA50'],
      top: 30
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true
    },
    xAxis: {
      type: 'category',
      data: chartData.value.map(item => item.date),
      scale: true,
      boundaryGap: false,
      axisLine: { onZero: false },
      splitLine: { show: false },
      min: 'dataMin',
      max: 'dataMax'
    },
    yAxis: {
      scale: true,
      splitArea: {
        show: true
      }
    },
    dataZoom: [
      {
        type: 'inside',
        start: 50,
        end: 100
      },
      {
        show: true,
        type: 'slider',
        top: '90%',
        start: 50,
        end: 100
      }
    ],
    series: [
      {
        name: 'K线',
        type: 'candlestick',
        data: chartData.value.map(item => [
          item.open,
          item.close,
          item.low,
          item.high
        ]),
        itemStyle: {
          color: '#ec0000',
          color0: '#00da3c',
          borderColor: '#8A0000',
          borderColor0: '#008F28'
        }
      },
      {
        name: 'MA20',
        type: 'line',
        data: chartData.value.map(item => item.ma20),
        smooth: true,
        lineStyle: {
          opacity: 0.8,
          width: 1
        }
      },
      {
        name: 'MA50',
        type: 'line',
        data: chartData.value.map(item => item.ma50),
        smooth: true,
        lineStyle: {
          opacity: 0.8,
          width: 1
        }
      }
    ]
  }
})

// 监听时间周期变化
watch(timeframe, () => {
  loadChartData()
})

// 加载图表数据
const loadChartData = async () => {
  try {
    // 这里应该调用真实的API获取K线数据
    // 暂时使用模拟数据
    chartData.value = generateMockChartData()
    calculateIndicators()
  } catch (error) {
    console.error('加载图表数据失败:', error)
  }
}

// 生成模拟K线数据
const generateMockChartData = () => {
  const data = []
  const basePrice = tradingStore.currentPrice || 45000
  let currentPrice = basePrice
  
  for (let i = 0; i < 100; i++) {
    const change = (Math.random() - 0.5) * 1000
    const open = currentPrice
    const close = open + change
    const high = Math.max(open, close) + Math.random() * 100
    const low = Math.min(open, close) - Math.random() * 100
    
    data.push({
      date: new Date(Date.now() - (99 - i) * 60 * 60 * 1000).toISOString(),
      open: open,
      high: high,
      low: low,
      close: close,
      volume: Math.random() * 1000
    })
    
    currentPrice = close
  }
  
  return data
}

// 计算技术指标
const calculateIndicators = () => {
  if (chartData.value.length === 0) return
  
  // 计算移动平均线
  const prices = chartData.value.map(item => item.close)
  ma20Value.value = calculateMA(prices, 20)
  ma50Value.value = calculateMA(prices, 50)
  
  // 计算RSI
  rsiValue.value = calculateRSI(prices, 14)
  
  // 计算MACD
  macdValue.value = calculateMACD(prices)
  
  // 更新图表数据中的MA值
  chartData.value.forEach((item, index) => {
    item.ma20 = calculateMA(prices.slice(0, index + 1), 20)
    item.ma50 = calculateMA(prices.slice(0, index + 1), 50)
  })
}

// 计算移动平均线
const calculateMA = (prices, period) => {
  if (prices.length < period) return 0
  const sum = prices.slice(-period).reduce((a, b) => a + b, 0)
  return sum / period
}

// 计算RSI
const calculateRSI = (prices, period) => {
  if (prices.length < period + 1) return 50
  
  let gains = 0
  let losses = 0
  
  for (let i = 1; i <= period; i++) {
    const change = prices[i] - prices[i - 1]
    if (change > 0) {
      gains += change
    } else {
      losses += Math.abs(change)
    }
  }
  
  const avgGain = gains / period
  const avgLoss = losses / period
  
  if (avgLoss === 0) return 100
  
  const rs = avgGain / avgLoss
  return 100 - (100 / (1 + rs))
}

// 计算MACD
const calculateMACD = (prices) => {
  if (prices.length < 26) return 0
  
  const ema12 = calculateEMA(prices, 12)
  const ema26 = calculateEMA(prices, 26)
  
  return ema12 - ema26
}

// 计算EMA
const calculateEMA = (prices, period) => {
  if (prices.length < period) return prices[prices.length - 1]
  
  const multiplier = 2 / (period + 1)
  let ema = prices[0]
  
  for (let i = 1; i < prices.length; i++) {
    ema = (prices[i] * multiplier) + (ema * (1 - multiplier))
  }
  
  return ema
}

// 获取RSI样式类
const getRSIClass = (rsi) => {
  if (rsi > 70) return 'overbought'
  if (rsi < 30) return 'oversold'
  return 'normal'
}

// 获取MACD样式类
const getMACDClass = (macd) => {
  if (macd > 0) return 'positive'
  if (macd < 0) return 'negative'
  return 'neutral'
}

// 格式化价格
const formatPrice = (price) => {
  if (!price) return '0.00'
  return parseFloat(price).toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })
}

// 刷新图表
const refreshChart = () => {
  loadChartData()
}

// 图表点击事件
const onChartClick = (params) => {
  console.log('图表点击:', params)
}

onMounted(() => {
  loadChartData()
})
</script>

<style scoped>
.trading-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.chart-card {
  flex: 1;
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.chart-title {
  display: flex;
  align-items: center;
  font-weight: bold;
}

.chart-title .el-icon {
  margin-right: 8px;
}

.chart-controls {
  display: flex;
  gap: 8px;
  align-items: center;
}

.chart-container {
  margin-top: 16px;
}

.indicators-card {
  margin-top: 16px;
}

.card-header {
  display: flex;
  align-items: center;
  font-weight: bold;
}

.card-header .el-icon {
  margin-right: 8px;
}

.indicators-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 16px;
}

.indicator-item {
  text-align: center;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 8px;
}

.indicator-label {
  font-size: 12px;
  color: #909399;
  margin-bottom: 8px;
}

.indicator-value {
  font-size: 18px;
  font-weight: bold;
  color: #303133;
}

.indicator-value.overbought {
  color: #f56c6c;
}

.indicator-value.oversold {
  color: #67c23a;
}

.indicator-value.positive {
  color: #67c23a;
}

.indicator-value.negative {
  color: #f56c6c;
}

.indicator-value.neutral {
  color: #909399;
}
</style>
