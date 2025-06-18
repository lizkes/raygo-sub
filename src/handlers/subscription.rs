use crate::handlers::common::{SubscriptionQuery, decrypt_secret, get_client_ip};
use crate::models::AppState;

use ntex::web::types::{Query, State};
use ntex::web::{HttpRequest, HttpResponse, Responder};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// 订阅请求处理函数
pub async fn handle_subscription(
    req: HttpRequest,
    query: Query<SubscriptionQuery>,
    state: State<AppState>,
) -> impl Responder {
    let client_ip = get_client_ip(&req);

    // 检查是否提供了secret参数
    let encrypted_secret = match &query.secret {
        Some(secret) => secret,
        None => {
            warn!("[{}] ❌ 缺少secret参数，访问被禁止", client_ip);
            return HttpResponse::NoContent().finish();
        }
    };

    let use_compression = query.zstd;

    // 解密secret获得uuid
    let uuid_str = match decrypt_secret(&encrypted_secret, &state.app_config.encryption_key) {
        Ok(decrypted) => {
            debug!("[{}] 🔓 成功解密secret获得uuid: {}", client_ip, decrypted);
            decrypted
        }
        Err(e) => {
            warn!("[{}] ❌ secret解密失败，访问被禁止: {}", client_ip, e);
            return HttpResponse::NoContent().finish();
        }
    };

    // 验证解密后的UUID格式
    let uuid = match Uuid::parse_str(&uuid_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            warn!(
                "[{}] ❌ 解密后的数据不是有效UUID，访问被禁止: {}",
                client_ip, uuid_str
            );
            return HttpResponse::NoContent().finish();
        }
    };

    if use_compression {
        info!("[{}] 📥 收到订阅请求(启用zstd压缩): {}", client_ip, uuid);
    } else {
        info!("[{}] 📥 收到订阅请求(不使用压缩): {}", client_ip, uuid);
    }

    // 处理订阅配置
    // 1. 获取配置读锁并克隆配置以便修改
    let clash_config = {
        let config_guard = state.clash_config.read().await;
        config_guard.clone()
    };
    let mut clash_config = clash_config;

    debug!("[{}] ✅ 使用缓存的配置文件", client_ip);

    // 2. 替换proxies中的uuid字段
    if let Some(ref mut proxies) = clash_config.proxies {
        let mut replaced_count = 0;
        for proxy in proxies.iter_mut() {
            if proxy.contains_key("uuid") {
                proxy.insert(
                    "uuid".to_string(),
                    serde_yaml_ng::Value::String(uuid.to_string()),
                );
                replaced_count += 1;
            }
        }
        debug!(
            "[{}] 🔄 替换了 {} 个代理的UUID为: {}",
            client_ip, replaced_count, uuid
        );
    } else {
        warn!("[{}] ⚠️  配置中没有找到proxies字段", client_ip);
    }

    // 3. 使用 serde_yaml_ng 将配置序列化为返回给用户的 YAML 字符串
    let yaml_body = match serde_yaml_ng::to_string(&clash_config) {
        Ok(yaml) => yaml,
        Err(e) => {
            error!("❌ 配置序列化失败: {}", e);
            return HttpResponse::NoContent().finish();
        }
    };

    let original_size = yaml_body.len();

    // 4. 根据查询参数决定是否压缩
    if use_compression {
        // 使用 zstd 压缩 YAML 内容
        let compressed_data = match zstd::encode_all(yaml_body.as_bytes(), 3) {
            Ok(data) => {
                let compressed_size = data.len();
                let compression_ratio =
                    (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;
                info!(
                    "[{}] ✅ 成功生成压缩订阅配置，原始大小: {} 字节，压缩后大小: {} 字节，压缩率: {:.1}%",
                    client_ip, original_size, compressed_size, compression_ratio
                );
                data
            }
            Err(e) => {
                error!("[{}] ❌ 压缩失败: {}", client_ip, e);
                return HttpResponse::NoContent().finish();
            }
        };

        // 返回压缩后的响应
        HttpResponse::Ok()
            .content_type("application/x-yaml; charset=utf-8")
            .header("Content-Encoding", "zstd")
            .header(
                "Content-Disposition",
                "attachment; filename=RayGo; filename*=UTF-8''RayGo%E8%AE%A2%E9%98%85",
            )
            .header("Cache-Control", "no-cache")
            .header("X-Original-Size", original_size.to_string())
            .body(compressed_data)
    } else {
        // 返回未压缩的响应
        info!(
            "[{}] ✅ 成功生成订阅配置，大小: {} 字节",
            client_ip, original_size
        );

        HttpResponse::Ok()
            .content_type("application/x-yaml; charset=utf-8")
            .header(
                "Content-Disposition",
                "attachment; filename=RayGo; filename*=UTF-8''RayGo%E8%AE%A2%E9%98%85",
            )
            .header("Cache-Control", "no-cache")
            .body(yaml_body)
    }
}
