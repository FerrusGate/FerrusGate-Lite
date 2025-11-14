use ferrusgate_lite::AppError;
use ferrusgate_lite::config::{args, init_config};
use ferrusgate_lite::runtime::{prepare_server, run_server};
use std::env;

#[actix_web::main]
async fn main() -> Result<(), AppError> {
    // 解析命令行参数获取配置文件路径
    let cli_args: Vec<String> = env::args().collect();
    let config_path = args::parse_config_path(&cli_args);

    // 初始化全局配置
    init_config(config_path);

    // 初始化服务器
    let ctx = prepare_server().await?;

    tracing::info!("FerrusGate-Lite is ready");

    // 启动 HTTP 服务器
    run_server(ctx)
        .await
        .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
