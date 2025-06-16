# ---- 构建阶段 ----
FROM rust:1.86-alpine as builder

# 设置工作目录
WORKDIR /app

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    binutils

# 复制项目配置文件
COPY Cargo.toml Cargo.lock ./

# 下载依赖项（不进行编译，避免虚拟二进制文件问题）
RUN echo "下载项目依赖..." && \
    cargo fetch && \
    echo "依赖下载完成"

# 复制源代码
COPY src ./src
COPY utils ./utils

# 构建项目
RUN echo "开始构建项目..." && \
    cargo build --release && \
    echo "构建完成，验证二进制文件:" && \
    ls -la target/release/raygo-* && \
    strip target/release/raygo-sub && \
    strip target/release/raygo-encrypt && \
    echo "项目构建和优化完成"

# ---- 运行阶段 ----
FROM alpine:latest

# 设置时区
ENV TZ=Asia/Shanghai

# 安装运行时依赖
RUN apk add --no-cache \
    ca-certificates \
    tzdata && \
    ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && \
    echo $TZ > /etc/timezone

# 设置工作目录
WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/raygo-sub /app/raygo-sub
COPY --from=builder /app/target/release/raygo-encrypt /app/raygo-encrypt

# 验证复制的文件
RUN echo "验证复制的二进制文件:" && \
    ls -la /app/raygo-* && \
    echo "文件验证完成"

# 暴露端口
EXPOSE 8080

# 启动服务
CMD ["./raygo-sub"]
