use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// DNS配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
    #[serde(rename = "cache-algorithm", skip_serializing_if = "Option::is_none")]
    pub cache_algorithm: Option<String>,
    #[serde(rename = "prefer-h3", skip_serializing_if = "Option::is_none")]
    pub prefer_h3: Option<bool>,
    #[serde(rename = "use-hosts", skip_serializing_if = "Option::is_none")]
    pub use_hosts: Option<bool>,
    #[serde(rename = "use-system-hosts", skip_serializing_if = "Option::is_none")]
    pub use_system_hosts: Option<bool>,
    #[serde(rename = "respect-rules", skip_serializing_if = "Option::is_none")]
    pub respect_rules: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<bool>,
    #[serde(rename = "enhanced-mode", skip_serializing_if = "Option::is_none")]
    pub enhanced_mode: Option<String>,
    #[serde(rename = "fake-ip-range", skip_serializing_if = "Option::is_none")]
    pub fake_ip_range: Option<String>,
    #[serde(
        rename = "fake-ip-filter-mode",
        skip_serializing_if = "Option::is_none"
    )]
    pub fake_ip_filter_mode: Option<String>,
    #[serde(rename = "fake-ip-filter", skip_serializing_if = "Option::is_none")]
    pub fake_ip_filter: Option<Vec<String>>,
    #[serde(rename = "nameserver-policy", skip_serializing_if = "Option::is_none")]
    pub nameserver_policy: Option<HashMap<String, serde_yaml_ng::Value>>,
    #[serde(rename = "default-nameserver", skip_serializing_if = "Option::is_none")]
    pub default_nameserver: Option<Vec<String>>,
    #[serde(
        rename = "proxy-server-nameserver",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_server_nameserver: Option<Vec<String>>,
    #[serde(rename = "direct-nameserver", skip_serializing_if = "Option::is_none")]
    pub direct_nameserver: Option<Vec<String>>,
    #[serde(
        rename = "direct-nameserver-follow-policy",
        skip_serializing_if = "Option::is_none"
    )]
    pub direct_nameserver_follow_policy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameserver: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Vec<String>>,
    #[serde(rename = "fallback-filter", skip_serializing_if = "Option::is_none")]
    pub fallback_filter: Option<DnsFallbackFilter>,
}

// DNS fallback过滤器配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsFallbackFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geoip: Option<bool>,
    #[serde(rename = "geoip-code", skip_serializing_if = "Option::is_none")]
    pub geoip_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipcidr: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geosite: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<String>>,
}

// TUN配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(rename = "auto-redirect", skip_serializing_if = "Option::is_none")]
    pub auto_redirect: Option<bool>,
    #[serde(
        rename = "auto-detect-interface",
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_detect_interface: Option<bool>,
    #[serde(rename = "dns-hijack", skip_serializing_if = "Option::is_none")]
    pub dns_hijack: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gso: Option<bool>,
    #[serde(rename = "gso-max-size", skip_serializing_if = "Option::is_none")]
    pub gso_max_size: Option<u32>,
    #[serde(rename = "udp-timeout", skip_serializing_if = "Option::is_none")]
    pub udp_timeout: Option<u32>,
    #[serde(rename = "auto-route", skip_serializing_if = "Option::is_none")]
    pub auto_route: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    #[serde(rename = "strict-route", skip_serializing_if = "Option::is_none")]
    pub strict_route: Option<bool>,
    #[serde(
        rename = "endpoint-independent-nat",
        skip_serializing_if = "Option::is_none"
    )]
    pub endpoint_independent_nat: Option<bool>,
    #[serde(rename = "include-interface", skip_serializing_if = "Option::is_none")]
    pub include_interface: Option<Vec<String>>,
    #[serde(rename = "exclude-interface", skip_serializing_if = "Option::is_none")]
    pub exclude_interface: Option<Vec<String>>,
    #[serde(rename = "include-uid", skip_serializing_if = "Option::is_none")]
    pub include_uid: Option<Vec<u32>>,
    #[serde(rename = "exclude-uid", skip_serializing_if = "Option::is_none")]
    pub exclude_uid: Option<Vec<u32>>,
    #[serde(
        rename = "include-android-user",
        skip_serializing_if = "Option::is_none"
    )]
    pub include_android_user: Option<Vec<u32>>,
    #[serde(rename = "include-package", skip_serializing_if = "Option::is_none")]
    pub include_package: Option<Vec<String>>,
    #[serde(rename = "exclude-package", skip_serializing_if = "Option::is_none")]
    pub exclude_package: Option<Vec<String>>,
}

// Profile配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileConfig {
    #[serde(rename = "store-selected", skip_serializing_if = "Option::is_none")]
    pub store_selected: Option<bool>,
    #[serde(rename = "store-fake-ip", skip_serializing_if = "Option::is_none")]
    pub store_fake_ip: Option<bool>,
}

// Geo数据配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeoxUrl {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geoip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geosite: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mmdb: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asn: Option<String>,
}

// 代理组类型枚举
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ProxyGroupType {
    Select,
    UrlTest,
    Fallback,
    LoadBalance,
    Relay,
}

// 代理组配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: ProxyGroupType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxies: Option<Vec<String>>,
    #[serde(rename = "use", skip_serializing_if = "Option::is_none")]
    pub use_providers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lazy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(rename = "max-failed-times", skip_serializing_if = "Option::is_none")]
    pub max_failed_times: Option<u32>,
    #[serde(rename = "disable-udp", skip_serializing_if = "Option::is_none")]
    pub disable_udp: Option<bool>,
    #[serde(rename = "interface-name", skip_serializing_if = "Option::is_none")]
    pub interface_name: Option<String>,
    #[serde(rename = "routing-mark", skip_serializing_if = "Option::is_none")]
    pub routing_mark: Option<u32>,
    #[serde(rename = "include-all", skip_serializing_if = "Option::is_none")]
    pub include_all: Option<bool>,
    #[serde(
        rename = "include-all-proxies",
        skip_serializing_if = "Option::is_none"
    )]
    pub include_all_proxies: Option<bool>,
    #[serde(
        rename = "include-all-providers",
        skip_serializing_if = "Option::is_none"
    )]
    pub include_all_providers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(rename = "exclude-filter", skip_serializing_if = "Option::is_none")]
    pub exclude_filter: Option<String>,
    #[serde(rename = "exclude-type", skip_serializing_if = "Option::is_none")]
    pub exclude_type: Option<String>,
    #[serde(rename = "expected-status", skip_serializing_if = "Option::is_none")]
    pub expected_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    // url-test 特有字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<u32>,
    // load-balance 特有字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

// 代理提供者类型枚举
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProxyProviderType {
    Http,
    File,
    Inline,
}

// 健康检查配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthCheck {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lazy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(rename = "max-failed-times", skip_serializing_if = "Option::is_none")]
    pub max_failed_times: Option<u32>,
    #[serde(rename = "expected-status", skip_serializing_if = "Option::is_none")]
    pub expected_status: Option<String>,
}

// 代理提供者配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyProvider {
    #[serde(rename = "type")]
    pub provider_type: ProxyProviderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(rename = "exclude-filter", skip_serializing_if = "Option::is_none")]
    pub exclude_filter: Option<String>,
    #[serde(rename = "exclude-type", skip_serializing_if = "Option::is_none")]
    pub exclude_type: Option<String>,
    #[serde(rename = "health-check", skip_serializing_if = "Option::is_none")]
    pub health_check: Option<HealthCheck>,
    #[serde(rename = "override", skip_serializing_if = "Option::is_none")]
    pub override_config: Option<HashMap<String, serde_yaml_ng::Value>>,
    // inline 类型特有字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxies: Option<Vec<HashMap<String, serde_yaml_ng::Value>>>,
}

// 规则类型枚举
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum RuleType {
    Domain,
    DomainSuffix,
    DomainKeyword,
    DomainRegex,
    Geosite,
    IpCidr,
    IpCidr6,
    IpSuffix,
    IpAsn,
    Geoip,
    SrcGeoip,
    SrcIpAsn,
    SrcIpCidr,
    SrcIpSuffix,
    DstPort,
    SrcPort,
    InPort,
    InType,
    InUser,
    InName,
    ProcessPath,
    ProcessPathRegex,
    ProcessName,
    ProcessNameRegex,
    Uid,
    Network,
    Dscp,
    RuleSet,
    And,
    Or,
    Not,
    SubRule,
    Match,
}

// 路由规则结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub rule_type: RuleType,
    pub payload: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<String>>, // 如 no-resolve, src 等参数
}

// 规则提供者类型枚举
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RuleProviderType {
    Http,
    File,
    Inline,
}

// 规则行为枚举
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RuleBehavior {
    Domain,
    Ipcidr,
    Classical,
}

// 规则格式枚举
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RuleFormat {
    Yaml,
    Text,
    Mrs,
}

// 规则提供者配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuleProvider {
    #[serde(rename = "type")]
    pub provider_type: RuleProviderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    pub behavior: RuleBehavior,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<RuleFormat>,
    #[serde(rename = "size-limit", skip_serializing_if = "Option::is_none")]
    pub size_limit: Option<u64>,
    // inline 类型特有字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Vec<String>>,
}

// 子规则配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubRule {
    pub rules: Vec<Rule>,
}

// 完整的Clash配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClashConfig {
    // 基础端口配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(rename = "socks-port", skip_serializing_if = "Option::is_none")]
    pub socks_port: Option<u16>,
    #[serde(rename = "mixed-port", skip_serializing_if = "Option::is_none")]
    pub mixed_port: Option<u16>,
    #[serde(rename = "redir-port", skip_serializing_if = "Option::is_none")]
    pub redir_port: Option<u16>,
    #[serde(rename = "tproxy-port", skip_serializing_if = "Option::is_none")]
    pub tproxy_port: Option<u16>,

    // 全局配置
    #[serde(rename = "allow-lan", skip_serializing_if = "Option::is_none")]
    pub allow_lan: Option<bool>,
    #[serde(rename = "bind-address", skip_serializing_if = "Option::is_none")]
    pub bind_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(rename = "log-level", skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<bool>,

    // 外部控制器
    #[serde(
        rename = "external-controller",
        skip_serializing_if = "Option::is_none"
    )]
    pub external_controller: Option<String>,
    #[serde(
        rename = "external-controller-cors",
        skip_serializing_if = "Option::is_none"
    )]
    pub external_controller_cors: Option<HashMap<String, serde_yaml_ng::Value>>,
    #[serde(
        rename = "external-controller-unix",
        skip_serializing_if = "Option::is_none"
    )]
    pub external_controller_unix: Option<String>,
    #[serde(
        rename = "external-controller-pipe",
        skip_serializing_if = "Option::is_none"
    )]
    pub external_controller_pipe: Option<String>,
    #[serde(
        rename = "external-controller-tls",
        skip_serializing_if = "Option::is_none"
    )]
    pub external_controller_tls: Option<String>,
    #[serde(
        rename = "external-doh-server",
        skip_serializing_if = "Option::is_none"
    )]
    pub external_doh_server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,

    // 用户界面
    #[serde(rename = "external-ui", skip_serializing_if = "Option::is_none")]
    pub external_ui: Option<String>,
    #[serde(rename = "external-ui-name", skip_serializing_if = "Option::is_none")]
    pub external_ui_name: Option<String>,
    #[serde(rename = "external-ui-url", skip_serializing_if = "Option::is_none")]
    pub external_ui_url: Option<String>,

    // 认证
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Vec<String>>,
    #[serde(rename = "skip-auth-prefixes", skip_serializing_if = "Option::is_none")]
    pub skip_auth_prefixes: Option<Vec<String>>,

    // 高级配置
    #[serde(rename = "unified-delay", skip_serializing_if = "Option::is_none")]
    pub unified_delay: Option<bool>,
    #[serde(rename = "tcp-concurrent", skip_serializing_if = "Option::is_none")]
    pub tcp_concurrent: Option<bool>,
    #[serde(rename = "interface-name", skip_serializing_if = "Option::is_none")]
    pub interface_name: Option<String>,
    #[serde(rename = "routing-mark", skip_serializing_if = "Option::is_none")]
    pub routing_mark: Option<u32>,
    #[serde(
        rename = "global-client-fingerprint",
        skip_serializing_if = "Option::is_none"
    )]
    pub global_client_fingerprint: Option<String>,
    #[serde(rename = "global-ua", skip_serializing_if = "Option::is_none")]
    pub global_ua: Option<String>,

    // GEO数据配置
    #[serde(rename = "geodata-mode", skip_serializing_if = "Option::is_none")]
    pub geodata_mode: Option<bool>,
    #[serde(rename = "geodata-loader", skip_serializing_if = "Option::is_none")]
    pub geodata_loader: Option<String>,
    #[serde(rename = "geo-auto-update", skip_serializing_if = "Option::is_none")]
    pub geo_auto_update: Option<bool>,
    #[serde(
        rename = "geo-update-interval",
        skip_serializing_if = "Option::is_none"
    )]
    pub geo_update_interval: Option<u32>,
    #[serde(rename = "geox-url", skip_serializing_if = "Option::is_none")]
    pub geox_url: Option<GeoxUrl>,

    // 进程匹配
    #[serde(rename = "find-process-mode", skip_serializing_if = "Option::is_none")]
    pub find_process_mode: Option<String>,

    // TLS配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<HashMap<String, serde_yaml_ng::Value>>,

    // DNS配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<DnsConfig>,

    // TUN配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tun: Option<TunConfig>,

    // Profile配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<ProfileConfig>,

    // hosts配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<HashMap<String, serde_yaml_ng::Value>>,

    // 代理配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxies: Option<Vec<HashMap<String, serde_yaml_ng::Value>>>,

    // 代理组配置 - 现在使用类型化结构体
    #[serde(rename = "proxy-groups", skip_serializing_if = "Option::is_none")]
    pub proxy_groups: Option<Vec<ProxyGroup>>,

    // 代理提供者配置 - 现在使用类型化结构体
    #[serde(rename = "proxy-providers", skip_serializing_if = "Option::is_none")]
    pub proxy_providers: Option<HashMap<String, ProxyProvider>>,

    // 规则配置 - 保持字符串格式以兼容现有配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,

    // 规则提供者配置 - 现在使用类型化结构体
    #[serde(rename = "rule-providers", skip_serializing_if = "Option::is_none")]
    pub rule_providers: Option<HashMap<String, RuleProvider>>,

    // 子规则配置 - 现在使用类型化结构体
    #[serde(rename = "sub-rules", skip_serializing_if = "Option::is_none")]
    pub sub_rules: Option<HashMap<String, SubRule>>,

    // 监听器配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listeners: Option<Vec<HashMap<String, serde_yaml_ng::Value>>>,

    // 隧道配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnels: Option<Vec<HashMap<String, serde_yaml_ng::Value>>>,

    // NTP配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ntp: Option<HashMap<String, serde_yaml_ng::Value>>,

    // 实验性功能
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_yaml_ng::Value>>,
}

impl Default for ClashConfig {
    fn default() -> Self {
        Self {
            port: None,
            socks_port: None,
            mixed_port: Some(7890),
            redir_port: None,
            tproxy_port: None,
            allow_lan: Some(false),
            bind_address: None,
            mode: Some("rule".to_string()),
            log_level: Some("info".to_string()),
            ipv6: Some(false),
            external_controller: None,
            external_controller_cors: None,
            external_controller_unix: None,
            external_controller_pipe: None,
            external_controller_tls: None,
            external_doh_server: None,
            secret: None,
            external_ui: None,
            external_ui_name: None,
            external_ui_url: None,
            authentication: None,
            skip_auth_prefixes: None,
            unified_delay: Some(true),
            tcp_concurrent: Some(true),
            interface_name: None,
            routing_mark: None,
            global_client_fingerprint: None,
            global_ua: None,
            geodata_mode: Some(false),
            geodata_loader: Some("memconservative".to_string()),
            geo_auto_update: Some(true),
            geo_update_interval: Some(24),
            geox_url: None,
            find_process_mode: None,
            tls: None,
            dns: None,
            tun: None,
            profile: None,
            hosts: None,
            proxies: None,
            proxy_groups: None,
            proxy_providers: None,
            rules: None,
            rule_providers: None,
            sub_rules: None,
            listeners: None,
            tunnels: None,
            ntp: None,
            experimental: None,
        }
    }
}

// 应用配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub addr: String,
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub encryption_key: String, // Base64编码的32字节密钥
    pub admin_password: String, // 管理员密码
}

// 默认日志级别
fn default_log_level() -> String {
    "info".to_string()
}

// 应用状态结构体，用于缓存配置文件内容
#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub app_config: AppConfig,
    pub clash_config: Arc<RwLock<ClashConfig>>,
}
