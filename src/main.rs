mod matching_engine;
mod orderbook;
mod simple_main;
mod types;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 使用简化版本运行
    simple_main::run_simple_server().await
}
