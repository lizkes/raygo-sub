use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use futures_util::StreamExt;
use ntex::web::HttpRequest;
use ntex_multipart::Multipart;
use serde::Deserialize;
use std::collections::HashMap;

// 查询参数结构体
#[derive(Deserialize)]
pub struct SubscriptionQuery {
    #[serde(default)]
    pub zstd: bool,
    pub secret: Option<String>, // 加密的UUID
}

// 配置管理查询参数结构体
#[derive(Deserialize)]
pub struct ConfigQuery {
    pub auth: Option<String>, // 加密的管理员密码
}

// Multipart表单数据结构
#[derive(Debug)]
pub struct ConfigFormData {
    pub auth_token: String,
    pub config_content: String,
}

// 解析multipart表单数据的辅助函数
pub async fn parse_multipart_form(mut multipart: Multipart) -> Result<ConfigFormData, String> {
    let mut fields = HashMap::new();

    while let Some(field_result) = multipart.next().await {
        let mut field = field_result.map_err(|e| format!("解析字段失败: {}", e))?;

        let field_name = field
            .headers()
            .get("content-disposition")
            .and_then(|cd| {
                let cd_str = std::str::from_utf8(cd.as_bytes()).ok()?;
                // 简单解析 Content-Disposition 中的 name 参数
                cd_str
                    .split(';')
                    .find(|part| part.trim().starts_with("name="))
                    .and_then(|name_part| {
                        let name = name_part.trim().strip_prefix("name=")?;
                        Some(name.trim_matches('"').to_string())
                    })
            })
            .unwrap_or_else(|| "unknown".to_string());

        let mut data = Vec::new();
        while let Some(chunk_result) = field.next().await {
            let chunk = chunk_result.map_err(|e| format!("读取字段数据失败: {}", e))?;
            data.extend_from_slice(&chunk);
        }

        let value = String::from_utf8(data)
            .map_err(|e| format!("字段{}不是有效的UTF-8: {}", field_name, e))?;

        fields.insert(field_name, value);
    }

    let auth_token = fields
        .get("auth_token")
        .ok_or("缺少auth_token字段")?
        .clone();

    let config_content = fields
        .get("config_content")
        .ok_or("缺少config_content字段")?
        .clone();

    Ok(ConfigFormData {
        auth_token,
        config_content,
    })
}

// ChaCha20Poly1305解密函数
pub fn decrypt_secret(encrypted_secret: &str, key_base64: &str) -> Result<String, String> {
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
pub fn get_client_ip(req: &HttpRequest) -> String {
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

// HTML转义函数
pub fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
