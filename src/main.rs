use ferrusgate_lite::AppError;
use ferrusgate_lite::config::{args, init_config};
use ferrusgate_lite::runtime::{listen_for_shutdown, prepare_server, run_server};
use ferrusgate_lite::system::install_panic_hook;
use std::env;

#[actix_web::main]
async fn main() -> Result<(), AppError> {
    // 安装 panic hook
    install_panic_hook();

    // 解析命令行参数获取配置文件路径
    let cli_args: Vec<String> = env::args().collect();
    let config_path = args::parse_config_path(&cli_args);

    // 初始化全局配置
    init_config(config_path);

    // 初始化服务器
    let ctx = prepare_server().await?;

    tracing::info!("FerrusGate-Lite is ready");

    // 启动 HTTP 服务器和优雅关闭监听
    tokio::select! {
        result = run_server(ctx) => {
            result.map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;
        }
        _ = listen_for_shutdown() => {
            tracing::info!("收到关闭信号，正在停止服务器...");
        }
    }

    Ok(())
}
