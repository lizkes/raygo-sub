use crate::models::AppState;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use log::{debug, error, info, warn};
use ntex::web::types::{Query, State};
use ntex::web::{HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

// 查询参数结构体
#[derive(Deserialize)]
pub struct SubscriptionQuery {
    #[serde(default)]
    pub zstd: bool,
    pub secret: Option<String>, // 加密的UUID
}

// ChaCha20Poly1305解密函数
fn decrypt_secret(encrypted_secret: &str, key_base64: &str) -> Result<String, String> {
    // 1. 解码Base64编码的密钥
    let key_bytes = BASE64
        .decode(key_base64)
        .map_err(|e| format!("密钥Base64解码失败: {}", e))?;

    if key_bytes.len() != 32 {
        return Err(format!(
            "密钥长度不正确，需要32字节，实际: {} 字节",
            key_bytes.len()
        ));
    }

    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);

    // 2. 还原URL安全的Base64编码并解码
    let normalized_data = encrypted_secret.replace('-', "+").replace('_', "/");

    // 补充缺失的填充字符
    let padded_data = match normalized_data.len() % 4 {
        0 => normalized_data,
        2 => format!("{}==", normalized_data),
        3 => format!("{}=", normalized_data),
        _ => return Err("无效的Base64编码格式".to_string()),
    };

    let encrypted_bytes = BASE64
        .decode(&padded_data)
        .map_err(|e| format!("加密数据Base64解码失败: {}", e))?;

    // 3. 分离nonce和密文
    // ChaCha20Poly1305的nonce是12字节，认证标签是16字节
    if encrypted_bytes.len() < 12 + 16 {
        return Err("加密数据太短".to_string());
    }

    let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // 4. 解密
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("解密失败: {}", e))?;

    // 5. 转换为UTF-8字符串
    String::from_utf8(plaintext).map_err(|e| format!("解密结果不是有效的UTF-8字符串: {}", e))
}

// 获取客户端真实IP地址
fn get_client_ip(req: &HttpRequest) -> String {
    // 优先检查代理头部 (X-Forwarded-For, X-Real-IP等)
    if let Some(xff) = req.headers().get("X-Forwarded-For") {
        if let Ok(xff_str) = xff.to_str() {
            // X-Forwarded-For 可能包含多个IP，取第一个
            if let Some(first_ip) = xff_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return real_ip_str.to_string();
        }
    }

    // 如果没有代理头部，使用连接信息
    if let Some(peer_addr) = req.peer_addr() {
        return peer_addr.ip().to_string();
    }

    "未知".to_string()
}

// 订阅请求处理函数
pub async fn handle_subscription_request(
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
            return HttpResponse::NoContent()
                .content_type("text/plain; charset=utf-8")
                .body("");
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
            return HttpResponse::NoContent()
                .content_type("text/plain; charset=utf-8")
                .body("");
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
            return HttpResponse::NoContent()
                .content_type("text/plain; charset=utf-8")
                .body("");
        }
    };

    if use_compression {
        info!("[{}] 📥 收到订阅请求(启用zstd压缩): {}", client_ip, uuid);
    } else {
        info!("[{}] 📥 收到订阅请求(不使用压缩): {}", client_ip, uuid);
    }

    // 处理订阅配置
    // 1. 克隆配置以便修改
    let mut clash_config = state.clash_config.clone();

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
            return HttpResponse::NoContent()
                .content_type("text/plain; charset=utf-8")
                .body("");
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
                return HttpResponse::NoContent()
                    .content_type("text/plain; charset=utf-8")
                    .body("");
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

// 非法路径请求
pub async fn handle_other(req: HttpRequest) -> impl Responder {
    let client_ip = get_client_ip(&req);

    warn!("[{}] ❌ 请求路径错误，访问被禁止: {}", client_ip, req.uri());

    HttpResponse::NoContent().finish()
}
