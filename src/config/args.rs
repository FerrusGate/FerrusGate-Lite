//! 命令行参数解析
//!
//! 提供从命令行参数中提取配置文件路径的工具

/// 从命令行参数解析配置文件路径
///
/// 支持多种格式:
/// - `-c path` / `--config path`
/// - `-c=path` / `--config=path`
///
/// # 参数
/// * `args` - 命令行参数（包含程序名在索引 0）
///
/// # 返回
/// * `Some(String)` - 找到配置文件路径
/// * `None` - 未指定配置文件
///
/// # 示例
/// ```
/// let args = vec!["program".to_string(), "-c".to_string(), "custom.toml".to_string()];
/// assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
/// ```
pub fn parse_config_path(args: &[String]) -> Option<String> {
    let mut i = 1; // 跳过程序名
    while i < args.len() {
        let arg = &args[i];

        // 检查 -c 或 --config 后带值
        if (arg == "-c" || arg == "--config") && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }

        // 检查 -c=value 或 --config=value
        if let Some(path) = arg.strip_prefix("-c=") {
            return Some(path.to_string());
        }
        if let Some(path) = arg.strip_prefix("--config=") {
            return Some(path.to_string());
        }

        i += 1;
    }

    None
}

/// 从参数列表中过滤掉配置相关参数
///
/// 移除 `-c`/`--config` 及其值,避免干扰模式检测
///
/// # 参数
/// * `args` - 原始命令行参数
///
/// # 返回
/// 移除配置参数后的新向量
///
/// # 示例
/// ```
/// let args = vec!["program", "-c", "custom.toml", "serve"];
/// let filtered = filter_config_args(&args);
/// // 结果: ["program", "serve"]
/// ```
pub fn filter_config_args(args: &[String]) -> Vec<String> {
    let mut filtered = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        // 跳过 -c 或 --config 及其后续值
        if (arg == "-c" || arg == "--config") && i + 1 < args.len() {
            i += 2;
            continue;
        }

        // 跳过 -c=value 或 --config=value
        if arg.starts_with("-c=") || arg.starts_with("--config=") {
            i += 1;
            continue;
        }

        // 保留该参数
        filtered.push(arg.clone());
        i += 1;
    }

    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_path_short_flag() {
        let args = vec![
            "program".to_string(),
            "-c".to_string(),
            "custom.toml".to_string(),
        ];
        assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
    }

    #[test]
    fn test_parse_config_path_long_flag() {
        let args = vec![
            "program".to_string(),
            "--config".to_string(),
            "custom.toml".to_string(),
        ];
        assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
    }

    #[test]
    fn test_parse_config_path_short_equals() {
        let args = vec!["program".to_string(), "-c=custom.toml".to_string()];
        assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
    }

    #[test]
    fn test_parse_config_path_long_equals() {
        let args = vec!["program".to_string(), "--config=custom.toml".to_string()];
        assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
    }

    #[test]
    fn test_parse_config_path_none() {
        let args = vec!["program".to_string(), "serve".to_string()];
        assert_eq!(parse_config_path(&args), None);
    }

    #[test]
    fn test_filter_config_args_short_flag() {
        let args = vec![
            "program".to_string(),
            "-c".to_string(),
            "custom.toml".to_string(),
            "serve".to_string(),
        ];
        let filtered = filter_config_args(&args);
        assert_eq!(filtered, vec!["program".to_string(), "serve".to_string()]);
    }

    #[test]
    fn test_filter_config_args_equals() {
        let args = vec![
            "program".to_string(),
            "--config=custom.toml".to_string(),
            "serve".to_string(),
        ];
        let filtered = filter_config_args(&args);
        assert_eq!(filtered, vec!["program".to_string(), "serve".to_string()]);
    }

    #[test]
    fn test_filter_config_args_no_config() {
        let args = vec!["program".to_string(), "serve".to_string()];
        let filtered = filter_config_args(&args);
        assert_eq!(filtered, vec!["program".to_string(), "serve".to_string()]);
    }
}
