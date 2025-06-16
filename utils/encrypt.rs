use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use std::env;
use std::fs;
use std::path::Path;
use uuid::Uuid;

// 从主模块导入AppConfig
mod models {
    include!("../src/models.rs");
}
use models::AppConfig;

// 读取配置文件
fn load_config() -> Result<AppConfig, String> {
    let config_path = "config/app.yml";

    if !Path::new(config_path).exists() {
        return Err(format!("配置文件不存在: {}", config_path));
    }

    let config_content =
        fs::read_to_string(config_path).map_err(|e| format!("无法读取配置文件: {}", e))?;

    let config: AppConfig =
        serde_yaml_ng::from_str(&config_content).map_err(|e| format!("配置文件解析失败: {}", e))?;

    Ok(config)
}

fn main() {
    // 读取配置文件获取encryption_key
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            println!("❌ {}", e);
            return;
        }
    };

    let (uuids, input_file_path) = get_uuids();

    if uuids.is_empty() {
        println!("❌ 没有提供任何UUID");
        return;
    }

    let mut results = Vec::new();

    for (_, uuid_str) in uuids.iter().enumerate() {
        let uuid_str = uuid_str.trim();

        if uuid_str.is_empty() {
            continue;
        }

        // 验证UUID格式
        if let Err(_) = Uuid::parse_str(uuid_str) {
            let error_msg = format!("无效的UUID格式");
            let result = EncryptionResult {
                uuid: uuid_str.to_string(),
                encrypted: None,
                error: Some(error_msg.clone()),
            };
            results.push(result);

            println!("❌ 无效的UUID格式: {}", uuid_str);
            continue;
        }

        match encrypt_uuid(uuid_str, &config.encryption_key) {
            Ok(encrypted) => {
                let result = EncryptionResult {
                    uuid: uuid_str.to_string(),
                    encrypted: Some(encrypted.clone()),
                    error: None,
                };
                results.push(result);

                // 只输出测试URL
                println!("{}/?secret={}", config.sub_url, encrypted);
            }
            Err(e) => {
                let result = EncryptionResult {
                    uuid: uuid_str.to_string(),
                    encrypted: None,
                    error: Some(e.clone()),
                };
                results.push(result);

                println!("❌ 加密失败: {} - {}", uuid_str, e);
            }
        }
    }

    // 如果是文件输入，将结果写入文件
    if let Some(file_path) = input_file_path {
        if let Err(e) = write_results_to_file(&results, &file_path, &config) {
            println!("❌ 保存文件失败: {}", e);
        }
    }
}

fn get_uuids() -> (Vec<String>, Option<String>) {
    let args: Vec<String> = env::args().collect();

    // 如果提供了文件路径参数，使用指定文件
    if args.len() > 1 {
        let file_path = &args[1];

        // 检查文件是否存在
        if std::path::Path::new(file_path).exists() {
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    let uuids: Vec<String> = content
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty() && !line.starts_with('#')) // 过滤空行和注释行
                        .collect();

                    return (uuids, Some(file_path.clone()));
                }
                Err(e) => {
                    println!("❌ 无法读取文件: {}", e);
                    return (Vec::new(), None);
                }
            }
        } else {
            println!("❌ 文件不存在: {}", file_path);
            return (Vec::new(), None);
        }
    }

    // 没有参数时，默认读取config/uuid文件
    let default_file = "config/uuid";

    if std::path::Path::new(default_file).exists() {
        match fs::read_to_string(default_file) {
            Ok(content) => {
                let uuids: Vec<String> = content
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty() && !line.starts_with('#')) // 过滤空行和注释行
                    .collect();

                (uuids, Some(default_file.to_string()))
            }
            Err(e) => {
                println!("❌ 无法读取默认UUID文件 {}: {}", default_file, e);
                (Vec::new(), None)
            }
        }
    } else {
        println!("❌ 默认UUID文件不存在: {}", default_file);
        println!("请创建 {} 文件或通过参数指定UUID文件路径", default_file);
        (Vec::new(), None)
    }
}

fn encrypt_uuid(uuid_str: &str, key_base64: &str) -> Result<String, String> {
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

    // 2. 生成随机nonce (12字节)
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut nonce_bytes = [0u8; 12];
    // 使用时间戳和UUID前几个字符生成nonce
    let uuid_bytes = uuid_str.as_bytes();
    for i in 0..12 {
        nonce_bytes[i] = ((timestamp >> (i * 5)) as u8) ^ uuid_bytes.get(i).unwrap_or(&0);
    }
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 3. 加密UUID字符串
    let plaintext = uuid_str.as_bytes();
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("加密失败: {}", e))?;

    // 4. 组合nonce和密文
    let mut encrypted_data = Vec::new();
    encrypted_data.extend_from_slice(&nonce_bytes);
    encrypted_data.extend_from_slice(&ciphertext);

    // 5. Base64编码并进行URL安全处理
    Ok(BASE64
        .encode(&encrypted_data)
        .replace('+', "-")
        .replace('/', "_")
        .replace('=', ""))
}

// 加密结果结构体
struct EncryptionResult {
    uuid: String,
    encrypted: Option<String>,
    error: Option<String>,
}

// 写入加密结果到文件
fn write_results_to_file(
    results: &[EncryptionResult],
    input_file_path: &str,
    config: &AppConfig,
) -> Result<(), String> {
    // 生成输出文件名
    let input_path = Path::new(input_file_path);
    let file_name = input_path
        .file_name()
        .ok_or("无法获取文件名")?
        .to_str()
        .ok_or("文件名包含无效字符")?;

    let output_file_name = format!("encrypted_{}", file_name);
    let output_path = input_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(&output_file_name);

    // 创建输出内容 - 简化格式，只显示测试URL
    let mut content = String::new();

    for result in results {
        match &result.encrypted {
            Some(encrypted) => {
                content.push_str(&format!("{}/?secret={}\n", config.sub_url, encrypted));
            }
            None => {
                if let Some(error) = &result.error {
                    content.push_str(&format!("# 错误: {} - {}\n", result.uuid, error));
                }
            }
        }
    }

    // 写入文件
    fs::write(&output_path, content).map_err(|e| format!("写入文件失败: {}", e))?;

    println!("结果已保存到文件: {}", output_path.display());
    Ok(())
}
