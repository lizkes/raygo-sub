// 通用模块
pub mod common;

// 订阅处理模块
pub mod subscription;

// 配置管理模块
pub mod config;

// 其他模块
pub mod other;

// 重新导出主要的handler函数，保持向后兼容
pub use config::{handle_config_get, handle_config_post, handle_config_reload};
pub use other::{handle_favicon, handle_other};
pub use subscription::handle_subscription;
