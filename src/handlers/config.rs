use crate::handlers::common::{
    ConfigQuery, decrypt_secret, get_client_ip, html_escape, parse_multipart_form,
};
use crate::models::AppState;
use ntex_multipart::Multipart;

use ntex::web::types::{Query, State};
use ntex::web::{HttpRequest, HttpResponse, Responder};
use tracing::{debug, error, info, warn};

// 配置编辑页面处理函数 (GET)
pub async fn handle_config_get(
    req: HttpRequest,
    query: Query<ConfigQuery>,
    state: State<AppState>,
) -> impl Responder {
    let client_ip = get_client_ip(&req);

    // 检查是否提供了auth参数
    let encrypted_auth = match &query.auth {
        Some(auth) => auth,
        None => {
            warn!("[{}] ❌ /config 缺少auth参数，访问被禁止", client_ip);
            return HttpResponse::NoContent().finish();
        }
    };

    // 解密auth获得管理员密码
    let decrypted_password = match decrypt_secret(&encrypted_auth, &state.app_config.encryption_key)
    {
        Ok(decrypted) => {
            debug!("[{}] 🔓 /config 成功解密auth", client_ip);
            decrypted
        }
        Err(e) => {
            warn!("[{}] ❌ /config auth解密失败，访问被禁止: {}", client_ip, e);
            return HttpResponse::NoContent().finish();
        }
    };

    // 验证解密后的密码是否与admin_password匹配
    if decrypted_password != state.app_config.admin_password {
        warn!("[{}] ❌ /config 管理员密码验证失败，访问被禁止", client_ip);
        return HttpResponse::NoContent().finish();
    }

    info!("[{}] 📝 管理员访问配置编辑页面", client_ip);

    // 读取当前配置文件内容
    let config_content = match tokio::fs::read_to_string("config/clash.yml").await {
        Ok(content) => content,
        Err(e) => {
            error!("[{}] ❌ 读取配置文件失败: {}", client_ip, e);
            return HttpResponse::InternalServerError()
                .content_type("text/html; charset=utf-8")
                .body("<h1>错误</h1><p>无法读取配置文件</p>");
        }
    };

    // 生成HTML编辑页面
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RayGo配置编辑器</title>
    <link rel="icon" href="/favicon.svg" type="image/svg+xml">
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            padding: 30px;
        }}
        h1 {{
            color: #333;
            text-align: center;
            margin-bottom: 30px;
        }}
        .form-group {{
            margin-bottom: 20px;
        }}
        label {{
            display: block;
            margin-bottom: 8px;
            font-weight: bold;
            color: #555;
        }}
        textarea {{
            width: 100%;
            height: 500px;
            padding: 15px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-family: 'Courier New', monospace;
            font-size: 14px;
            line-height: 1.5;
            resize: vertical;
            box-sizing: border-box;
        }}
        .button-group {{
            text-align: center;
            margin-top: 20px;
        }}
        button {{
            background-color: #007bff;
            color: white;
            padding: 12px 30px;
            border: none;
            border-radius: 4px;
            font-size: 16px;
            cursor: pointer;
            margin: 0 10px;
        }}
        button:hover {{
            background-color: #0056b3;
        }}
        .reset-btn {{
            background-color: #6c757d;
        }}
        .reset-btn:hover {{
            background-color: #545b62;
        }}
        .info {{
            background-color: #e7f3ff;
            border: 1px solid #bee5eb;
            border-radius: 4px;
            padding: 15px;
            margin-bottom: 20px;
        }}
        .warning {{
            background-color: #fff3cd;
            border: 1px solid #ffeaa7;
            border-radius: 4px;
            padding: 15px;
            margin-bottom: 20px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 RayGo配置编辑器</h1>
        
        <div class="info">
            <strong>📋 使用说明：</strong>
            <ul>
                <li>保存后会自动重载配置到内存</li>
                <li>请确保YAML格式正确，否则可能导致服务异常</li>
            </ul>
        </div>
        
        <form method="POST" action="/config" id="configForm" enctype="multipart/form-data">
            <input type="hidden" name="auth_token" value="{}">
            <div class="form-group">
                <label for="config_content">配置内容 (config/clash.yml)：</label>
                <textarea name="config_content" id="config_content" required style="height: 98vh">{}</textarea>
            </div>
            <div class="button-group">
                <button type="submit">💾 保存配置</button>
                <button type="button" class="reset-btn" onclick="location.reload()">🔄 重置</button>
            </div>
        </form>
    </div>

    <script>
        // 简单的保存确认和Authorization Bearer处理
        document.querySelector('form').addEventListener('submit', function(e) {{
            if (!confirm('确定要保存配置吗？这将重启配置服务。')) {{
                e.preventDefault();
                return;
            }}
            
            // 使用fetch API发送带有Authorization头的请求
            e.preventDefault();
            
            const formData = new FormData(this);
            const authToken = formData.get('auth_token');
            
            fetch('/config', {{
                method: 'POST',
                headers: {{
                    'Authorization': `Bearer ${{authToken}}`
                }},
                body: formData
            }})
            .then(response => {{
                if (!response.ok) {{
                    return response.text().then(text => {{
                        throw new Error(`HTTP ${{response.status}}: ${{text}}`);
                    }});
                }}
                return response.text();
            }})
            .then(html => {{
                document.open();
                document.write(html);
                document.close();
            }})
            .catch(error => {{
                console.error('保存配置失败:', error);
                alert('保存配置时发生错误:\\n\\n' + error.message + '\\n\\n请查看控制台获取更多信息');
            }});
        }});
        
        // Ctrl+S 快捷键保存
        document.addEventListener('keydown', function(e) {{
            if (e.ctrlKey && e.key === 's') {{
                e.preventDefault();
                if (confirm('确定要保存配置吗？')) {{
                    document.querySelector('form').dispatchEvent(new Event('submit'));
                }}
            }}
        }});
    </script>
</body>
</html>"#,
        encrypted_auth,
        html_escape(&config_content)
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_content)
}

// 配置保存处理函数 (POST)
pub async fn handle_config_post(
    req: HttpRequest,
    multipart: Multipart,
    state: State<AppState>,
) -> impl Responder {
    let client_ip = get_client_ip(&req);

    // 解析multipart表单数据
    let form_data = match parse_multipart_form(multipart).await {
        Ok(data) => data,
        Err(e) => {
            warn!(
                "[{}] ❌ /config POST 解析multipart数据失败: {}",
                client_ip, e
            );
            return HttpResponse::BadRequest()
                .content_type("text/html; charset=utf-8")
                .body(&format!(
                    "<h1>错误</h1><p>表单数据解析失败: {}</p>",
                    html_escape(&e)
                ));
        }
    };

    // 验证Authorization Bearer头（优先使用）
    let auth_token = if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                token
            } else {
                &form_data.auth_token
            }
        } else {
            &form_data.auth_token
        }
    } else {
        // 回退到表单中的auth_token
        &form_data.auth_token
    };

    // 解密token获得管理员密码
    let decrypted_password = match decrypt_secret(auth_token, &state.app_config.encryption_key) {
        Ok(decrypted) => {
            debug!("[{}] 🔓 /config POST 成功解密token", client_ip);
            decrypted
        }
        Err(e) => {
            warn!(
                "[{}] ❌ /config POST token解密失败，访问被禁止: {}",
                client_ip, e
            );
            return HttpResponse::BadRequest()
                .content_type("text/html; charset=utf-8")
                .body("<h1>错误</h1><p>身份验证失败</p>");
        }
    };

    // 验证解密后的密码是否与admin_password匹配
    if decrypted_password != state.app_config.admin_password {
        warn!(
            "[{}] ❌ /config POST 管理员密码验证失败，访问被禁止",
            client_ip
        );
        return HttpResponse::BadRequest()
            .content_type("text/html; charset=utf-8")
            .body("<h1>错误</h1><p>身份验证失败</p>");
    }

    info!("[{}] 💾 管理员开始保存配置", client_ip);

    // 验证新配置格式
    let new_config: crate::models::ClashConfig =
        match serde_yaml_ng::from_str(&form_data.config_content) {
            Ok(config) => config,
            Err(e) => {
                warn!("[{}] ❌ 配置格式验证失败: {}", client_ip, e);
                return HttpResponse::BadRequest()
                    .content_type("text/html; charset=utf-8")
                    .body(&format!(
                        r#"<h1>配置格式错误</h1>
                    <p>YAML解析失败: {}</p>
                    <p><a href="javascript:history.back()">返回修改</a></p>"#,
                        html_escape(&e.to_string())
                    ));
            }
        };

    // 保存配置到文件
    if let Err(e) = tokio::fs::write("config/clash.yml", &form_data.config_content).await {
        error!("[{}] ❌ 保存配置文件失败: {}", client_ip, e);
        return HttpResponse::InternalServerError()
            .content_type("text/html; charset=utf-8")
            .body("<h1>保存失败</h1><p>无法写入配置文件</p>");
    }

    // 更新内存中的配置
    {
        let mut config_guard = state.clash_config.write().await;
        *config_guard = new_config;
    }

    info!("[{}] ✅ 配置保存并重载成功", client_ip);

    // 返回成功页面
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(&format!(
            r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>保存成功 - RayGo配置编辑器</title>
    <link rel="icon" href="/favicon.svg" type="image/svg+xml">
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
            text-align: center;
        }}
        .container {{
            max-width: 600px;
            margin: 100px auto;
            background-color: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            padding: 40px;
        }}
        .success {{
            color: #28a745;
            font-size: 24px;
            margin-bottom: 20px;
        }}
        .btn {{
            background-color: #007bff;
            color: white;
            padding: 12px 30px;
            text-decoration: none;
            border-radius: 4px;
            display: inline-block;
            margin: 10px;
        }}
        .btn:hover {{
            background-color: #0056b3;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="success">✅ 配置保存成功！</div>
        <p>配置已成功保存到 config/clash.yml 并重载到内存。</p>
        <p>所有后续订阅请求将使用新配置。</p>
        <div>
            <a href="/config?auth={}" class="btn">🔄 继续编辑</a>
        </div>
    </div>
</body>
</html>"#,
            form_data.auth_token
        ))
}

// 热重载处理函数
pub async fn handle_config_reload(req: HttpRequest, state: State<AppState>) -> impl Responder {
    let client_ip = get_client_ip(&req);

    // 验证Authorization Bearer头
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header,
        None => {
            warn!(
                "[{}] ❌ /config/reload 缺少Authorization头，访问被禁止",
                client_ip
            );
            return HttpResponse::NoContent().finish();
        }
    };

    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            warn!("[{}] ❌ /config/reload Authorization头格式无效", client_ip);
            return HttpResponse::NoContent().finish();
        }
    };

    // 检查Bearer前缀
    let token = if auth_str.starts_with("Bearer ") {
        &auth_str[7..] // 去掉"Bearer "前缀
    } else {
        warn!(
            "[{}] ❌ /config/reload Authorization头缺少Bearer前缀",
            client_ip
        );
        return HttpResponse::NoContent().finish();
    };

    // 验证token是否是加密后的值
    let decrypt_plaintext = match decrypt_secret(token, &state.app_config.encryption_key) {
        Ok(decrypted) => decrypted,
        Err(e) => {
            warn!("[{}] ❌ /config/reload token解密失败: {}", client_ip, e);
            return HttpResponse::NoContent().finish();
        }
    };

    if decrypt_plaintext != state.app_config.admin_password {
        warn!(
            "[{}] ❌ /config/reload token验证失败，与管理员密码不符，实际'{}'",
            client_ip, decrypt_plaintext
        );
        return HttpResponse::NoContent().finish();
    }

    info!("[{}] 🔄 开始热重载配置文件", client_ip);

    // 重新读取clash.yml配置文件
    let clash_config_content = match tokio::fs::read_to_string("config/clash.yml").await {
        Ok(content) => content,
        Err(e) => {
            error!(
                "[{}] ❌ 热重载失败: 无法读取config/clash.yml - {}",
                client_ip, e
            );
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body("Failed to read config file");
        }
    };

    // 解析新的配置文件
    let new_clash_config = match serde_yaml_ng::from_str(&clash_config_content) {
        Ok(config) => config,
        Err(e) => {
            error!(
                "[{}] ❌ 热重载失败: config/clash.yml解析错误 - {}",
                client_ip, e
            );
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body("Failed to parse config file");
        }
    };

    // 更新配置
    {
        let mut config_guard = state.clash_config.write().await;
        *config_guard = new_clash_config;
    }

    info!("[{}] ✅ 配置文件热重载成功", client_ip);

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("配置重载成功")
}
