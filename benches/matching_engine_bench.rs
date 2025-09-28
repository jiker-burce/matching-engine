use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use matching_engine::matching_engine::MatchingEngine;
use matching_engine::orderbook::OrderBook;
use matching_engine::types::*;
use std::sync::Arc;
use std::time::Duration;

/// 基准测试：订单提交性能
fn bench_order_submission(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_submission");
    group.measurement_time(Duration::from_secs(10));

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("submit_orders", size), size, |b, &size| {
            let engine = Arc::new(MatchingEngine::new());
            let symbol = Symbol::new("BTC", "USDT");

            b.iter(|| {
                for i in 0..*size {
                    let order = Order::new(
                        symbol.clone(),
                        if i % 2 == 0 {
                            OrderSide::Buy
                        } else {
                            OrderSide::Sell
                        },
                        OrderType::Limit,
                        1.0,
                        Some(50000.0 + (i as f64)),
                        format!("user_{}", i),
                    );

                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let _ = engine.submit_order(order).await;
                    });
                }
            });
        });
    }
    group.finish();
}

/// 基准测试：订单簿操作性能
fn bench_orderbook_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook_operations");

    // 测试添加订单
    group.bench_function("add_order", |b| {
        let mut orderbook = OrderBook::new(Symbol::new("BTC", "USDT"));

        b.iter(|| {
            let order = Order::new(
                Symbol::new("BTC", "USDT"),
                OrderSide::Buy,
                OrderType::Limit,
                1.0,
                Some(50000.0),
                "user".to_string(),
            );
            black_box(orderbook.add_order(order).unwrap());
        });
    });

    // 测试获取最佳价格
    group.bench_function("get_best_prices", |b| {
        let mut orderbook = OrderBook::new(Symbol::new("BTC", "USDT"));

        // 预填充一些订单
        for i in 0..1000 {
            let order = Order::new(
                Symbol::new("BTC", "USDT"),
                if i % 2 == 0 {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                },
                OrderType::Limit,
                1.0,
                Some(50000.0 + (i as f64)),
                format!("user_{}", i),
            );
            orderbook.add_order(order).unwrap();
        }

        b.iter(|| {
            black_box(orderbook.best_bid());
            black_box(orderbook.best_ask());
            black_box(orderbook.spread());
        });
    });

    // 测试获取订单簿深度
    group.bench_function("get_depth", |b| {
        let mut orderbook = OrderBook::new(Symbol::new("BTC", "USDT"));

        // 预填充一些订单
        for i in 0..1000 {
            let order = Order::new(
                Symbol::new("BTC", "USDT"),
                if i % 2 == 0 {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                },
                OrderType::Limit,
                1.0,
                Some(50000.0 + (i as f64)),
                format!("user_{}", i),
            );
            orderbook.add_order(order).unwrap();
        }

        b.iter(|| {
            black_box(orderbook.get_depth(Some(10)));
        });
    });

    group.finish();
}

/// 基准测试：撮合性能
fn bench_matching_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("matching_performance");
    group.measurement_time(Duration::from_secs(15));

    for size in [100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("match_orders", size), size, |b, &size| {
            let engine = Arc::new(MatchingEngine::new());
            let symbol = Symbol::new("BTC", "USDT");

            // 预填充卖单
            let rt = tokio::runtime::Runtime::new().unwrap();
            for i in 0..*size {
                let sell_order = Order::new(
                    symbol.clone(),
                    OrderSide::Sell,
                    OrderType::Limit,
                    1.0,
                    Some(50000.0 + (i as f64)),
                    format!("seller_{}", i),
                );
                rt.block_on(async {
                    let _ = engine.submit_order(sell_order).await;
                });
            }

            b.iter(|| {
                // 提交买单进行撮合
                for i in 0..*size {
                    let buy_order = Order::new(
                        symbol.clone(),
                        OrderSide::Buy,
                        OrderType::Limit,
                        1.0,
                        Some(50000.0 + (i as f64) + 100.0), // 确保能匹配
                        format!("buyer_{}", i),
                    );

                    rt.block_on(async {
                        let _ = engine.submit_order(buy_order).await;
                    });
                }
            });
        });
    }
    group.finish();
}

/// 基准测试：并发性能
fn bench_concurrent_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_performance");
    group.measurement_time(Duration::from_secs(20));

    for num_threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_orders", num_threads),
            num_threads,
            |b, &num_threads| {
                let engine = Arc::new(MatchingEngine::new());
                let symbol = Symbol::new("BTC", "USDT");

                b.iter(|| {
                    let handles: Vec<_> = (0..*num_threads)
                        .map(|thread_id| {
                            let engine = engine.clone();
                            let symbol = symbol.clone();

                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Runtime::new().unwrap();

                                for i in 0..100 {
                                    let order = Order::new(
                                        symbol.clone(),
                                        if (thread_id + i) % 2 == 0 {
                                            OrderSide::Buy
                                        } else {
                                            OrderSide::Sell
                                        },
                                        OrderType::Limit,
                                        1.0,
                                        Some(50000.0 + (thread_id as f64 * 100.0) + (i as f64)),
                                        format!("user_{}_{}", thread_id, i),
                                    );

                                    rt.block_on(async {
                                        let _ = engine.submit_order(order).await;
                                    });
                                }
                            })
                        })
                        .collect();

                    // 等待所有线程完成
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }
    group.finish();
}

/// 基准测试：内存使用
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    group.bench_function("large_orderbook", |b| {
        b.iter(|| {
            let mut orderbook = OrderBook::new(Symbol::new("BTC", "USDT"));

            // 创建大量订单
            for i in 0..10000 {
                let order = Order::new(
                    Symbol::new("BTC", "USDT"),
                    if i % 2 == 0 {
                        OrderSide::Buy
                    } else {
                        OrderSide::Sell
                    },
                    OrderType::Limit,
                    1.0,
                    Some(50000.0 + (i as f64)),
                    format!("user_{}", i),
                );
                orderbook.add_order(order).unwrap();
            }

            black_box(orderbook.get_stats());
        });
    });

    group.finish();
}

/// 基准测试：序列化性能
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    // 测试订单序列化
    group.bench_function("serialize_order", |b| {
        let order = Order::new(
            Symbol::new("BTC", "USDT"),
            OrderSide::Buy,
            OrderType::Limit,
            1.0,
            Some(50000.0),
            "user".to_string(),
        );

        b.iter(|| {
            black_box(serde_json::to_string(&order).unwrap());
        });
    });

    // 测试订单反序列化
    group.bench_function("deserialize_order", |b| {
        let order = Order::new(
            Symbol::new("BTC", "USDT"),
            OrderSide::Buy,
            OrderType::Limit,
            1.0,
            Some(50000.0),
            "user".to_string(),
        );
        let json = serde_json::to_string(&order).unwrap();

        b.iter(|| {
            black_box(serde_json::from_str::<Order>(&json).unwrap());
        });
    });

    // 测试交易序列化
    group.bench_function("serialize_trade", |b| {
        let trade = Trade {
            id: uuid::Uuid::new_v4(),
            symbol: Symbol::new("BTC", "USDT"),
            buy_order_id: uuid::Uuid::new_v4(),
            sell_order_id: uuid::Uuid::new_v4(),
            quantity: 1.0,
            price: 50000.0,
            timestamp: chrono::Utc::now(),
            buyer_id: "buyer".to_string(),
            seller_id: "seller".to_string(),
        };

        b.iter(|| {
            black_box(serde_json::to_string(&trade).unwrap());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_order_submission,
    bench_orderbook_operations,
    bench_matching_performance,
    bench_concurrent_performance,
    bench_memory_usage,
    bench_serialization
);

criterion_main!(benches);
