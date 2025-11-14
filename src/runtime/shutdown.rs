use tokio::signal;
use tracing::warn;

/// 监听关闭信号并执行清理操作
pub async fn listen_for_shutdown() {
    // 等待 Ctrl+C 信号
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    warn!("收到关闭信号，正在执行清理操作...");

    // TODO: 这里可以添加其他清理逻辑
    // 例如：刷新缓存、关闭数据库连接等

    warn!("清理完成，即将关闭...");
}
