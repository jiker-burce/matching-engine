import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import axios from 'axios'
import dayjs from 'dayjs'
import MarketDataService from '../services/MarketDataService.js'

export const useTradingStore = defineStore('trading', () => {
  // 状态
  const websocketConnected = ref(false)
  const currentSymbol = ref('BTC/USDT')
  const marketData = ref({})
  const orderBook = ref({ bids: [], asks: [] })
  const trades = ref([])
  const myOrders = ref([])
  const priceData = ref([])
  const currentDataSource = ref('未知')
  
  // 初始化市场数据服务
  const marketDataService = new MarketDataService(axios)
  
  // 计算属性
  const currentPrice = computed(() => {
    return marketData.value.price || 0
  })
  
  const priceChange = computed(() => {
    return marketData.value.price_change_24h || 0
  })
  
  const priceChangePercent = computed(() => {
    return marketData.value.price_change_percentage_24h || 0
  })
  
  const volume24h = computed(() => {
    return marketData.value.total_volume || 0
  })
  
  // WebSocket连接
  let ws = null
  
  // 初始化
  const initialize = async () => {
    await loadMarketData()
    await loadOrderBook()
    await loadTradeHistory()
    connectWebSocket()
  }
  
  // 加载市场数据（使用策略模式）
  const loadMarketData = async () => {
    try {
      console.log('🚀 开始加载市场数据...')
      marketData.value = await marketDataService.getMarketData()
      
      // 记录当前使用的数据源
      currentDataSource.value = marketDataService.getLastUsedStrategy()
      
      console.log(`🎉 市场数据加载完成! 数据源: ${currentDataSource.value}`)
    } catch (error) {
      console.error('💥 获取市场数据失败:', error)
      currentDataSource.value = 'Default Data (Simulated)'
      // 使用默认数据作为最后的备用方案
      console.log('🔄 使用备用默认数据...')
      marketData.value = {
        price: 45000,
        price_change_24h: 0,
        price_change_percentage_24h: 0,
        total_volume: 25000000000,
        high_24h: 46000,
        low_24h: 44000,
        timestamp: new Date().toISOString()
      }
    }
  }
  
  // 加载订单簿数据
  const loadOrderBook = async () => {
    try {
      console.log('正在加载订单簿数据...')
      const response = await axios.get('/api/orderbook/BTC-USDT', {
        timeout: 3000
      })
      orderBook.value = response.data
      console.log('订单簿数据加载成功:', orderBook.value)
    } catch (error) {
      console.warn('订单簿API加载失败，使用模拟数据:', error.message)
      // 使用模拟数据
      orderBook.value = generateMockOrderBook()
      console.log('使用模拟订单簿数据:', orderBook.value)
    }
  }
  
  // 加载交易历史
  const loadTradeHistory = async () => {
    try {
      console.log('正在加载交易历史...')
      const response = await axios.get('/api/trades/BTC-USDT?limit=50', {
        timeout: 3000
      })
      trades.value = response.data
      console.log('交易历史加载成功:', trades.value.length, '条记录')
    } catch (error) {
      console.warn('交易历史API加载失败，使用模拟数据:', error.message)
      // 使用模拟数据
      trades.value = generateMockTrades()
      console.log('使用模拟交易数据:', trades.value.length, '条记录')
    }
  }
  
  // 连接WebSocket
  const connectWebSocket = () => {
    try {
      console.log('正在连接WebSocket...')
      ws = new WebSocket('ws://localhost:8888/ws')
      
      ws.onopen = () => {
        websocketConnected.value = true
        console.log('✅ WebSocket连接已建立')
      }
      
      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          console.log('收到WebSocket消息:', data)
          handleWebSocketMessage(data)
        } catch (error) {
          console.error('解析WebSocket消息失败:', error)
        }
      }
      
      ws.onclose = (event) => {
        websocketConnected.value = false
        console.log('❌ WebSocket连接已关闭:', event.code, event.reason)
        // 尝试重连
        console.log('5秒后尝试重连...')
        setTimeout(connectWebSocket, 5000)
      }
      
      ws.onerror = (error) => {
        console.error('❌ WebSocket错误:', error)
        websocketConnected.value = false
      }
    } catch (error) {
      console.error('❌ WebSocket连接失败:', error)
      websocketConnected.value = false
    }
  }
  
  // 处理WebSocket消息
  const handleWebSocketMessage = (data) => {
    switch (data.type) {
      case 'trade':
        trades.value.unshift(data.trade)
        if (trades.value.length > 100) {
          trades.value = trades.value.slice(0, 100)
        }
        break
      case 'orderbook':
        orderBook.value = data.orderbook
        break
      case 'market_data':
        marketData.value = data.market_data
        break
    }
  }
  
  // 提交订单
  const submitOrder = async (orderData) => {
    try {
      const response = await axios.post('/api/orders', orderData)
      if (response.data.success) {
        myOrders.value.unshift(response.data.order)
        return { success: true, data: response.data }
      }
    } catch (error) {
      console.error('提交订单失败:', error)
      return { success: false, error: error.message }
    }
  }
  
  // 取消订单
  const cancelOrder = async (orderId) => {
    try {
      await axios.delete(`/api/orders/${orderId}`)
      myOrders.value = myOrders.value.filter(order => order.id !== orderId)
      return { success: true }
    } catch (error) {
      console.error('取消订单失败:', error)
      return { success: false, error: error.message }
    }
  }
  
  // 断开连接
  const disconnect = () => {
    if (ws) {
      ws.close()
      ws = null
    }
  }
  
  // 生成模拟订单簿数据
  const generateMockOrderBook = () => {
    const currentPrice = marketData.value.price || 45000
    const bids = []
    const asks = []
    
    for (let i = 0; i < 10; i++) {
      const bidPrice = currentPrice - (i + 1) * 10
      const askPrice = currentPrice + (i + 1) * 10
      const quantity = Math.random() * 5 + 0.1
      
      bids.push({
        price: bidPrice,
        quantity: quantity,
        total: bidPrice * quantity
      })
      
      asks.push({
        price: askPrice,
        quantity: quantity,
        total: askPrice * quantity
      })
    }
    
    return { bids, asks }
  }
  
  // 生成模拟交易数据
  const generateMockTrades = () => {
    const trades = []
    const currentPrice = marketData.value.price || 45000
    
    for (let i = 0; i < 20; i++) {
      const price = currentPrice + (Math.random() - 0.5) * 1000
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
  
  return {
    // 状态
    websocketConnected,
    currentSymbol,
    marketData,
    orderBook,
    trades,
    myOrders,
    priceData,
    currentDataSource,
    
    // 计算属性
    currentPrice,
    priceChange,
    priceChangePercent,
    volume24h,
    
    // 方法
    initialize,
    loadMarketData,
    loadOrderBook,
    loadTradeHistory,
    submitOrder,
    cancelOrder,
    disconnect
  }
})
