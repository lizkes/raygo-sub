use log::info;
use ntex::web::{self, App, HttpServer};

mod handlers;
mod models;

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
    let app_state = models::AppState {
        app_config: app_config.clone(),
        clash_config,
    };

    // 根据配置文件初始化日志系统
    let log_level = match app_config.log_level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => {
            eprintln!(
                "⚠️  无效的日志级别: {}，使用默认级别 info",
                app_config.log_level
            );
            log::LevelFilter::Info
        }
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp_millis()
        .init();

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
            .default_service(web::route().to(handlers::handle_other))
    })
    .bind((app_config.addr.as_str(), app_config.port))?
    .run()
    .await
}
