/**
 * 市场数据服务
 * 使用责任链模式管理多个数据源
 */

import { 
  BackendApiStrategy, 
  BinanceApiStrategy, 
  CoinGeckoApiStrategy, 
  DefaultDataStrategy 
} from './DataSourceStrategy.js'

class MarketDataService {
  constructor(axios) {
    this.axios = axios
    this.strategies = [
      new BackendApiStrategy(axios),
      new BinanceApiStrategy(axios),
      new CoinGeckoApiStrategy(axios)
    ]
    this.lastUsedStrategy = null
  }
  
  /**
   * 获取市场数据
   * 按优先级尝试各个数据源
   */
  async getMarketData() {
    console.log('🔄 正在加载市场数据...')
    
    for (const strategy of this.strategies) {
      try {
        console.log(`🔍 尝试使用 ${strategy.getName()}...`)
        const data = await strategy.fetchData()
        
        // 记录成功使用的策略
        this.lastUsedStrategy = strategy
        
        // 根据数据源类型显示不同的成功信息
        if (strategy.getName() === 'Default Data (Simulated)') {
          console.log(`📊 使用模拟数据 (${strategy.getName()}) - 价格: $${data.price}`)
        } else {
          console.log(`🌐 使用真实API数据 (${strategy.getName()}) - 价格: $${data.price}`)
        }
        
        console.log(`✅ 数据源: ${strategy.getName()}`, {
          price: data.price,
          change24h: data.price_change_24h,
          volume: data.total_volume
        })
        
        return data
      } catch (error) {
        console.log(`❌ ${strategy.getName()} 失败:`, error.message)
        continue
      }
    }
    
    // 所有真实API都失败了
    throw new Error('服务器繁忙，请稍后再试')
  }
  
  /**
   * 添加新的数据源策略
   */
  addStrategy(strategy) {
    this.strategies.splice(-1, 0, strategy) // 在DefaultDataStrategy之前插入
  }
  
  /**
   * 移除数据源策略
   */
  removeStrategy(strategyName) {
    this.strategies = this.strategies.filter(s => s.getName() !== strategyName)
  }
  
  /**
   * 获取所有可用的数据源
   */
  getAvailableStrategies() {
    return this.strategies.map(s => s.getName())
  }
  
  /**
   * 获取最后使用的数据源
   */
  getLastUsedStrategy() {
    return this.lastUsedStrategy ? this.lastUsedStrategy.getName() : '未知'
  }
}

export default MarketDataService
