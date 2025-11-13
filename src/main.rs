use ferrusgate_lite::{AppConfig, AppError};
use ferrusgate_lite::runtime::{prepare_server, run_server};

#[actix_web::main]
async fn main() -> Result<(), AppError> {
    // 加载配置
    let config = AppConfig::load()?;

    // 初始化服务器
    let ctx = prepare_server(config).await?;

    tracing::info!("🚀 FerrusGate-Lite is ready!");

    // 启动 HTTP 服务器
    run_server(ctx).await
        .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
