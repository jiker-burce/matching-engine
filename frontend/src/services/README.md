# 服务层设计模式

## 概述

本项目使用了多种设计模式来管理数据源和业务逻辑，确保代码的可维护性和扩展性。

## 设计模式

### 1. 策略模式 (Strategy Pattern)

**文件**: `DataSourceStrategy.js`

**用途**: 封装不同的市场数据获取策略

**优势**:
- 每个数据源都有独立的实现
- 易于添加新的数据源
- 符合开闭原则（对扩展开放，对修改关闭）

**使用示例**:
```javascript
// 创建新的数据源策略
class CustomApiStrategy extends DataSourceStrategy {
  async fetchData() {
    // 自定义数据获取逻辑
  }
  
  getName() {
    return 'Custom API'
  }
}

// 添加到服务中
marketDataService.addStrategy(new CustomApiStrategy(axios))
```

### 2. 责任链模式 (Chain of Responsibility Pattern)

**文件**: `MarketDataService.js`

**用途**: 按优先级管理多个数据源，实现优雅降级

**优势**:
- 自动尝试多个数据源
- 失败时自动切换到下一个
- 集中管理数据源优先级

**数据源优先级**:
1. Backend API (最高优先级)
2. Binance API
3. CoinGecko API
4. Default Data (最低优先级，总是成功)

## 架构优势

### 1. 单一职责原则
- 每个策略只负责一种数据源的获取
- 服务类只负责协调各个策略

### 2. 开闭原则
- 添加新数据源不需要修改现有代码
- 只需创建新的策略类

### 3. 依赖倒置原则
- 高层模块不依赖低层模块
- 都依赖于抽象（DataSourceStrategy接口）

### 4. 可测试性
- 每个策略可以独立测试
- 可以轻松模拟不同的数据源

## 扩展指南

### 添加新的数据源

1. 创建新的策略类：
```javascript
class NewApiStrategy extends DataSourceStrategy {
  async fetchData() {
    // 实现数据获取逻辑
  }
  
  getName() {
    return 'New API'
  }
}
```

2. 注册到服务中：
```javascript
marketDataService.addStrategy(new NewApiStrategy(axios))
```

### 修改数据源优先级

```javascript
// 移除某个数据源
marketDataService.removeStrategy('CoinGecko API')

// 重新排序（通过重新创建服务实例）
const newService = new MarketDataService(axios)
newService.addStrategy(new CustomStrategy(axios))
newService.addStrategy(new BinanceApiStrategy(axios))
```

## 错误处理

每个策略都会抛出异常，服务会自动捕获并尝试下一个策略。这确保了：

- 单个数据源失败不会影响整个系统
- 用户总是能看到数据（即使是默认数据）
- 详细的日志记录便于调试

## 性能考虑

- 每个策略都有独立的超时设置
- 失败时立即切换到下一个策略
- 避免长时间等待不可用的数据源
