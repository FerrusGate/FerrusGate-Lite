//! Panic handler module
//!
//! 提供基于运行模式的 panic 处理策略:
//! - Server 模式: 显示详细堆栈跟踪，记录到 crash.log

use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;
use std::panic;

/// 安装自定义 panic hook
pub fn install_panic_hook() {
    let _default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "Unknown location".to_string());

        let backtrace = std::backtrace::Backtrace::force_capture();
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

        // 写入 crash.log
        if let Err(e) = write_crash_log(&timestamp, &message, &location, &backtrace) {
            eprintln!("Failed to write crash log: {}", e);
        }

        // Server 模式: 显示详细堆栈跟踪
        display_server_panic(&message, &location, &backtrace);
    }));
}

/// Server 模式: 显示详细的彩色堆栈跟踪信息
fn display_server_panic(message: &str, location: &str, backtrace: &std::backtrace::Backtrace) {
    use colored::Colorize;

    eprintln!();
    eprintln!(
        "{}",
        "═══════════════════════════════════════════════════"
            .red()
            .bold()
    );
    eprintln!("{}", "PANIC".red().bold());
    eprintln!(
        "{}",
        "═══════════════════════════════════════════════════"
            .red()
            .bold()
    );
    eprintln!();
    eprintln!("{} {}", "原因:".yellow().bold(), message.white());
    eprintln!("{} {}", "位置:".yellow().bold(), location.white());
    eprintln!();
    eprintln!("{}", "堆栈跟踪:".yellow().bold());
    eprintln!("{}", format!("{:?}", backtrace).dimmed());
    eprintln!();
    eprintln!("{}", "详细信息已保存到 crash.log".cyan());
    eprintln!(
        "{}",
        "═══════════════════════════════════════════════════"
            .red()
            .bold()
    );
    eprintln!();
}

/// 写入崩溃日志
fn write_crash_log(
    timestamp: &str,
    message: &str,
    location: &str,
    backtrace: &std::backtrace::Backtrace,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("crash.log")?;

    writeln!(file, "==========================================")?;
    writeln!(file, "Crash Report - {}", timestamp)?;
    writeln!(file, "==========================================")?;
    writeln!(file, "Message: {}", message)?;
    writeln!(file, "Location: {}", location)?;
    writeln!(file, "\nBacktrace:")?;
    writeln!(file, "{:?}", backtrace)?;
    writeln!(file, "==========================================\n")?;

    Ok(())
}
