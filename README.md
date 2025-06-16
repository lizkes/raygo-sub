# RayGo-Sub

基于 Rust 开发的高性能 Clash 订阅服务器，支持UUID加密验证和配置动态替换。

## 功能特性

- ⚡ **高性能**: 基于 ntex 框架，支持高并发访问
- 🔐 **安全加密**: 使用 ChaCha20Poly1305 加密算法保护UUID
- 📦 **智能压缩**: 支持 zstd 压缩减少流量消耗  
- 🚀 **内存缓存**: 配置文件预加载到内存，响应速度极快
- 📝 **详细日志**: 支持多级别日志记录和客户端IP追踪
- 🐳 **容器化**: 完整的 Docker 支持

## 项目结构

```
raygo-sub/
├── src/                 # 主服务源码
│   ├── main.rs         # 服务器入口
│   ├── handlers.rs     # 请求处理逻辑  
│   └── models.rs       # 数据模型定义
├── utils/              # 工具程序
│   └── encrypt.rs      # UUID加密工具
├── config/             # 配置文件目录
│   ├── app.yml         # 应用配置
│   ├── clash.yml       # Clash模板配置
│   └── uuid            # UUID列表(可选)
├── Dockerfile          # Docker构建文件
├── docker-compose.yml  # Docker Compose配置
└── Cargo.toml          # 项目依赖配置
```

## 快速开始

### 使用 Docker (推荐)

#### 方式一：使用 Docker Compose

```bash
# 克隆项目并进入目录
git clone <项目地址>
cd raygo-sub

## 修改app_example.yml、clash_example.yml、uuid_example，并更名去掉_example

# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f
```

#### 方式二：手动 Docker 运行

```bash
# 构建镜像
docker build -t raygo-sub .

# 运行容器
docker run -d \
  --name raygo-sub \
  -p 8080:8080 \
  -v ./config:/app/config:ro \
  raygo-sub

# 查看日志
docker logs -f raygo-sub
```

#### 方式三：使用预构建镜像

```bash
# 直接使用 GitHub Container Registry 镜像
docker run -d \
  --name raygo-sub \
  -p 8080:8080 \
  -v ./config:/app/config:ro \
  ghcr.io/lizkes/raygo-sub:latest
```

### 本地编译运行

```bash
# 安装 Rust 环境
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone <项目地址>
cd raygo-sub

# 编译运行主服务
cargo run --release --bin raygo-sub

# 编译加密工具
cargo build --release --bin raygo-encrypt
```

## 配置说明

### app.yml - 应用配置

```yaml
addr: 127.0.0.1              # 监听地址
port: 8080                  # 监听端口
log_level: info              # 日志级别: error/warn/info/debug/trace
encryption_key: "base64密钥"  # ChaCha20Poly1305 32字节密钥
sub_url: "http://127.0.0.1:8080"  # 订阅服务URL
```

### clash.yml - Clash模板配置

标准的 Clash 配置文件，其中的 `uuid` 字段会被动态替换。

### uuid - UUID列表文件

```
# 这是注释行，会被忽略
550e8400-e29b-41d4-a716-446655440000
6ba7b810-9dad-11d1-80b4-00c04fd430c8
# 支持空行和注释
```

## 使用方法

### 1. 生成加密的UUID

#### Docker环境中使用加密工具

```bash
# 进入容器
docker exec -it raygo-sub sh

# 使用加密工具 (读取config/uuid文件)
./raygo-encrypt

# 或指定文件
./raygo-encrypt /path/to/uuid-file
```

#### 本地环境使用加密工具

```bash
# 编译加密工具
cargo build --release --bin raygo-encrypt

# 使用工具生成加密URL (默认读取config/uuid)
./target/release/raygo-encrypt

# 指定UUID文件
./target/release/raygo-encrypt path/to/your/uuid-file
```

输出示例：
```
http://127.0.0.1:8080/?secret=SGVsbG9Xb3JsZA
```

### 2. 获取订阅配置

```bash
# 普通请求
curl "http://127.0.0.1:8080/?secret=SGVsbG9Xb3JsZA"

# 启用压缩
curl "http://127.0.0.1:8080/?secret=SGVsbG9Xb3JsZA&zstd=true"
```

### 3. 在 Clash 中使用

直接将生成的URL添加到 Clash 客户端的订阅列表中。

## Docker 详细配置

### 镜像说明

- **主服务**: `raygo-sub` - Web订阅服务器
- **工具程序**: `raygo-encrypt` - UUID加密工具
- **暴露端口**: 8080 (实际端口在配置文件中设置)
- **配置目录**: `/app/config` (需要挂载外部配置)

### 配置文件挂载

容器需要挂载以下配置文件：

```
config/
├── app.yml          # 应用配置 (必需)
├── clash.yml        # Clash模板配置 (必需)
└── uuid             # UUID列表文件 (可选，用于加密工具)
```

### 端口配置

- **容器内端口**: 由 `config/app.yml` 中的 `port` 配置决定
- **Docker端口映射**: 在 `docker-compose.yml` 或运行命令中配置
- **默认端口**: 8080

### 访问服务

- **默认订阅**: `http://localhost:8080/?secret=XXXXXX`
- **压缩订阅**: `http://localhost:8080/?secret=XXXXXX&zstd=true`

## API 接口

### GET /?secret=XXXX

获取个性化的 Clash 配置文件

**参数:**
- `secret` (必需): 加密的UUID字符串
- `zstd` (可选): 是否启用zstd压缩 (true/false)

**响应:**
- 成功: 返回YAML格式的Clash配置文件
- 失败: 返回403 Forbidden

## 开发说明

### 编译选项

项目支持编译两个程序：

```bash
# 编译主服务
cargo build --release --bin raygo-sub

# 编译加密工具  
cargo build --release --bin raygo-encrypt

# 编译所有程序
cargo build --release
```

### 日志级别

通过 `app.yml` 中的 `log_level` 配置：
- `error`: 只显示错误信息
- `warn`: 显示警告和错误
- `info`: 显示基本运行信息 (推荐)
- `debug`: 显示详细调试信息
- `trace`: 显示所有跟踪信息

### 查看日志

```bash
# Docker Compose
docker-compose logs -f

# Docker 容器
docker logs -f raygo-sub

# 本地运行
cargo run --release --bin raygo-sub
```

## 安全注意事项

1. **密钥安全**: 妥善保管 `encryption_key`，泄露后需要重新生成所有加密URL
2. **网络安全**: 建议在生产环境中使用 HTTPS
3. **访问控制**: 考虑配置防火墙限制访问来源
4. **日志安全**: 生产环境建议使用 `info` 或更高日志级别
5. **配置权限**: 确保配置文件挂载为只读模式 (`:ro`)

## 故障排除

### 常见问题

1. **容器启动失败**
   - 检查配置文件是否正确挂载
   - 确认 `app.yml` 和 `clash.yml` 文件存在且格式正确

2. **无法访问服务**
   - 检查端口映射是否正确
   - 确认防火墙设置
   - 查看容器日志排查问题

3. **加密工具无法使用**
   - 确认容器中存在 `raygo-encrypt` 文件
   - 检查 `config/uuid` 文件是否存在且格式正确

### 调试模式

```bash
# 启用debug日志
# 修改 config/app.yml 中的 log_level: debug

# 重启服务查看详细日志
docker-compose restart
docker-compose logs -f
```

## 许可证

MIT License 