mod handlers;
mod models;

use ntex::web::{self, App, HttpServer};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::fmt::time::OffsetTime;

use crate::models::AppState;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    // 读取应用配置文件
    let config_content = match tokio::fs::read_to_string("config/app.yml").await {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ 无法读取配置文件 config/app.yml: {}", e);
            return Err(e);
        }
    };

    // 解析应用配置文件
    let app_config: models::AppConfig = match serde_yaml_ng::from_str(&config_content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ 配置文件格式错误: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("配置文件解析失败: {}", e),
            ));
        }
    };

    // 读取Clash配置文件
    let clash_config_content = match tokio::fs::read_to_string("config/clash.yml").await {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ 无法读取配置文件 config/clash.yml: {}", e);
            return Err(e);
        }
    };

    // 解析Clash配置文件
    let clash_config: models::ClashConfig = match serde_yaml_ng::from_str(&clash_config_content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ Clash配置文件格式错误: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Clash配置解析失败: {}", e),
            ));
        }
    };

    // 创建应用状态
    let app_state = AppState {
        app_config: app_config.clone(),
        clash_config: Arc::new(RwLock::new(clash_config)),
    };

    // 根据配置文件初始化日志系统
    let log_level = match app_config.log_level.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => {
            eprintln!(
                "⚠️  无效的日志级别: {}，使用默认级别 info",
                app_config.log_level
            );
            tracing::Level::INFO
        }
    };

    // 先初始化tracing subscriber - 使用RFC 3339本地时间戳格式
    let offset = time::UtcOffset::current_local_offset().expect("无法获取本地时区偏移量");
    let time_format = time::format_description::parse(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]",
    )
    .expect("时间格式字符串无效");
    let timer = OffsetTime::new(offset, time_format);
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_timer(timer)
        .with_ansi(true)
        .init();

    // 初始化log桥接以捕获第三方库日志
    tracing_log::LogTracer::init_with_filter(tracing_log::log::LevelFilter::Warn)
        .expect("设置logger失败");

    info!("📝 日志级别设置为: {}", app_config.log_level);
    info!("🚀 RayGo-sub 服务器已启动");
    info!("🗃️  配置文件已加载到内存缓存");
    info!(
        "📍 服务地址: http://{}:{}",
        app_config.addr, app_config.port
    );
    info!("   - GET /?secret=XXXX - 获取对应的clash订阅文件");

    HttpServer::new(move || {
        App::new()
            .state(app_state.clone())
            .route("/", web::get().to(handlers::handle_subscription_request))
            .route("/reload", web::post().to(handlers::handle_reload))
            .default_service(web::route().to(handlers::handle_other))
    })
    .bind((app_config.addr.as_str(), app_config.port))?
    .run()
    .await
}
