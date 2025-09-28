/**
 * 数据源策略模式
 * 用于封装不同的市场数据获取策略
 */

// 基础策略接口
class DataSourceStrategy {
  async fetchData() {
    throw new Error('fetchData method must be implemented')
  }
  
  getName() {
    throw new Error('getName method must be implemented')
  }
}

// 后端API策略
class BackendApiStrategy extends DataSourceStrategy {
  constructor(axios) {
    super()
    this.axios = axios
  }
  
  async fetchData() {
    console.log('🔗 调用后端API: /market_data/BTC-USDT')
    const response = await this.axios.get('/market_data/BTC-USDT', {
      timeout: 3000
    })
    
    if (response.data && response.data.price) {
      console.log('✅ 后端API响应成功:', response.data)
      return {
        price: response.data.price,
        price_change_24h: response.data.price_change_24h || 0,
        price_change_percentage_24h: response.data.price_change_percentage_24h || 0,
        total_volume: response.data.total_volume || 0,
        high_24h: response.data.high_24h || response.data.price * 1.05,
        low_24h: response.data.low_24h || response.data.price * 0.95,
        timestamp: response.data.timestamp || new Date().toISOString()
      }
    }
    throw new Error('Backend API返回数据格式错误')
  }
  
  getName() {
    return 'Backend API'
  }
}

// Binance API策略
class BinanceApiStrategy extends DataSourceStrategy {
  constructor(axios) {
    super()
    this.axios = axios
  }
  
  async fetchData() {
    console.log('🔗 调用Binance API: https://api.binance.com/api/v3/ticker/24hr?symbol=BTCUSDT')
    const response = await this.axios.get('https://api.binance.com/api/v3/ticker/24hr?symbol=BTCUSDT', {
      timeout: 5000,
      headers: {
        'Accept': 'application/json'
      }
    })
    
    if (response.data && response.data.lastPrice) {
      console.log('✅ Binance API响应成功:', {
        price: response.data.lastPrice,
        change: response.data.priceChange
      })
      return {
        price: parseFloat(response.data.lastPrice),
        price_change_24h: parseFloat(response.data.priceChange),
        price_change_percentage_24h: parseFloat(response.data.priceChangePercent),
        total_volume: parseFloat(response.data.volume),
        high_24h: parseFloat(response.data.highPrice),
        low_24h: parseFloat(response.data.lowPrice),
        timestamp: new Date().toISOString()
      }
    }
    throw new Error('Binance API返回数据格式错误')
  }
  
  getName() {
    return 'Binance API'
  }
}

// CoinGecko API策略
class CoinGeckoApiStrategy extends DataSourceStrategy {
  constructor(axios) {
    super()
    this.axios = axios
  }
  
  async fetchData() {
    console.log('🔗 调用CoinGecko API: https://api.coingecko.com/api/v3/simple/price')
    const response = await this.axios.get('https://api.coingecko.com/api/v3/simple/price', {
      params: {
        ids: 'bitcoin',
        vs_currencies: 'usd',
        include_24hr_change: true,
        include_24hr_vol: true
      },
      timeout: 5000
    })
    
    const btcData = response.data.bitcoin
    if (btcData && btcData.usd) {
      console.log('✅ CoinGecko API响应成功:', {
        price: btcData.usd,
        change: btcData.usd_24h_change
      })
      return {
        price: btcData.usd,
        price_change_24h: btcData.usd_24h_change || 0,
        price_change_percentage_24h: btcData.usd_24h_change || 0,
        total_volume: btcData.usd_24h_vol || 0,
        high_24h: btcData.usd * 1.05,
        low_24h: btcData.usd * 0.95,
        timestamp: new Date().toISOString()
      }
    }
    throw new Error('CoinGecko API返回数据格式错误')
  }
  
  getName() {
    return 'CoinGecko API'
  }
}

// 默认数据策略
class DefaultDataStrategy extends DataSourceStrategy {
  fetchData() {
    console.log('📊 使用模拟数据 (Default Data) - 价格: $45000')
    return {
      price: 45000,
      price_change_24h: 0,
      price_change_percentage_24h: 0,
      total_volume: 25000000000,
      high_24h: 46000,
      low_24h: 44000,
      timestamp: new Date().toISOString()
    }
  }
  
  getName() {
    return 'Default Data'
  }
}

export { DataSourceStrategy, BackendApiStrategy, BinanceApiStrategy, CoinGeckoApiStrategy, DefaultDataStrategy }
