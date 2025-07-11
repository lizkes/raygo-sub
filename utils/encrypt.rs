use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::Path;

// 独立的应用配置结构
#[derive(Debug, Deserialize)]
struct AppConfig {
    encryption_key: String,
}

// 命令行参数结构
#[derive(Parser)]
#[command(name = "raygo-encrypt")]
#[command(about = "数据加密工具")]
struct Args {
    /// 配置文件路径
    #[arg(short = 'c', long = "config", default_value = "config/app.yml")]
    config_path: String,

    /// 直接指定加密密钥（优先级高于配置文件）
    #[arg(short = 's', long = "secret")]
    encryption_key: Option<String>,

    /// 数据文件路径
    #[arg(short = 'd', long = "data")]
    data_file: Option<String>,

    /// 要加密的字符串（优先级高于数据文件）
    input_string: Option<String>,
}

// 读取配置文件
fn load_config(config_path: &str) -> Result<AppConfig, String> {
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
    let args = Args::parse();

    // 获取加密密钥：优先使用-s参数，否则从配置文件读取
    let encryption_key = if let Some(key) = &args.encryption_key {
        // 使用命令行提供的密钥
        key.clone()
    } else {
        // 从配置文件读取密钥
        let config = match load_config(&args.config_path) {
            Ok(config) => config,
            Err(e) => {
                println!("❌ {}", e);
                return;
            }
        };
        config.encryption_key
    };

    // 根据参数决定数据来源（字符串优先级高于文件）
    if let Some(input_string) = &args.input_string {
        // 使用直接字符串模式（优先级最高）
        process_string(input_string, &encryption_key);
    } else if let Some(data_file_path) = &args.data_file {
        // 使用数据文件模式
        process_data_file(data_file_path, &encryption_key);
    } else {
        // 尝试使用默认数据文件
        let default_file = "config/data";
        if Path::new(default_file).exists() {
            process_data_file(default_file, &encryption_key);
        } else {
            println!("❌ 未提供输入数据");
            println!("请使用以下方式之一：");
            println!("  1. 直接传入字符串: raygo-encrypt \"要加密的内容\"");
            println!("  2. 使用数据文件: raygo-encrypt -d /path/to/data.txt");
            println!("  3. 创建默认数据文件: {}", default_file);
        }
    }
}

// 处理字符串输入
fn process_string(input_string: &str, encryption_key: &str) {
    match encrypt_data(input_string, encryption_key) {
        Ok(encrypted) => {
            println!("{}", encrypted);
        }
        Err(e) => {
            println!("❌ 加密失败: {}", e);
        }
    }
}

// 处理数据文件
fn process_data_file(file_path: &str, encryption_key: &str) {
    // 检查文件是否存在
    if !Path::new(file_path).exists() {
        println!("❌ 数据文件不存在: {}", file_path);
        return;
    }

    // 读取文件内容
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("❌ 无法读取数据文件: {}", e);
            return;
        }
    };

    let data_list: Vec<String> = content.lines().map(|line| line.to_string()).collect();

    if data_list.is_empty() {
        println!("❌ 数据文件为空");
        return;
    }

    let mut results = Vec::new();

    for data_str in data_list.iter() {
        let trimmed = data_str.trim();

        // 如果是空行或注释行，原样保留
        if trimmed.is_empty() || trimmed.starts_with('#') {
            let result = EncryptionResult {
                data: data_str.clone(),
                encrypted: None,
                error: None,
            };
            results.push(result);

            // 终端输出原行
            println!("{}", data_str);
            continue;
        }

        // 对普通数据行进行加密
        match encrypt_data(trimmed, encryption_key) {
            Ok(encrypted) => {
                let result = EncryptionResult {
                    data: data_str.clone(),
                    encrypted: Some(encrypted.clone()),
                    error: None,
                };
                results.push(result);

                // 直接输出加密后的数据
                println!("{}", encrypted);
            }
            Err(e) => {
                let result = EncryptionResult {
                    data: data_str.clone(),
                    encrypted: None,
                    error: Some(e.clone()),
                };
                results.push(result);

                println!("❌ 加密失败: {} - {}", trimmed, e);
            }
        }
    }

    // 将结果写入文件
    if let Err(e) = write_results_to_file(&results, file_path) {
        println!("❌ 保存文件失败: {}", e);
    }
}

fn encrypt_data(data_str: &str, key_base64: &str) -> Result<String, String> {
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
    // 使用时间戳和数据前几个字符生成nonce
    let data_bytes = data_str.as_bytes();
    for i in 0..12 {
        nonce_bytes[i] = ((timestamp >> (i * 5)) as u8) ^ data_bytes.get(i).unwrap_or(&0);
    }
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 3. 加密数据字符串
    let plaintext = data_str.as_bytes();
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
    data: String,
    encrypted: Option<String>,
    error: Option<String>,
}

// 写入加密结果到文件
fn write_results_to_file(
    results: &[EncryptionResult],
    input_file_path: &str,
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

    // 创建输出内容 - 保持原始格式
    let mut content = String::new();

    for result in results {
        match &result.encrypted {
            Some(encrypted) => {
                // 普通数据行，输出加密结果
                content.push_str(&format!("{}\n", encrypted));
            }
            None => {
                if let Some(error) = &result.error {
                    // 加密失败的行，输出错误信息
                    content.push_str(&format!("# 错误: {} - {}\n", result.data.trim(), error));
                } else {
                    // 空行或注释行，原样输出
                    content.push_str(&format!("{}\n", result.data));
                }
            }
        }
    }

    // 写入文件
    fs::write(&output_path, content).map_err(|e| format!("写入文件失败: {}", e))?;

    println!("结果已保存到文件: {}", output_path.display());
    Ok(())
}
