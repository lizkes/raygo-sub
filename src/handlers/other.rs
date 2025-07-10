use crate::handlers::common::get_client_ip;

use ntex::web::{HttpRequest, HttpResponse, Responder};
use tracing::debug;

// 内嵌的 favicon SVG 内容
const FAVICON_SVG: &str = r#"<svg viewBox="0 0 48 48" xmlns="http://www.w3.org/2000/svg"><defs><style>.a,.b{fill:none;stroke:#000;stroke-linecap:round}.a{stroke-linejoin:round}.b{stroke-miterlimit:5.7143}</style></defs><path class="a" d="M27.19 42.5a89.044 89.044 0 0 1-14.681-1.573S13.94 12.373 17.92 5.537c-.13-.297 2.992 1.212 4.422 6.266a25.557 25.557 0 0 1 4.847-.47"/><ellipse class="a" cx="21.24" cy="20.309" rx="1.671" ry="2.13"/><path class="a" d="M27.19 42.5a89.044 89.044 0 0 0 14.681-1.573S40.44 12.373 36.458 5.537c.03-.2-3.59 1.755-4.421 6.266a25.558 25.558 0 0 0-4.848-.47"/><ellipse class="a" cx="33.14" cy="20.309" rx="1.671" ry="2.13"/><path class="b" d="M12.508 40.927c-1.93-.327-4.948-.31-6.04-3.487-1.067-3.107.438-6.67 3.742-7.045M25.463 26.387a1.467 1.467 0 0 0 1.473-1.472M28.41 26.387a1.467 1.467 0 0 1-1.474-1.472"/></svg>"#;

// 非法路径请求
pub async fn handle_other(req: HttpRequest) -> impl Responder {
    let client_ip = get_client_ip(&req);

    debug!("[{}] 请求路径错误，访问被禁止: {}", client_ip, req.uri());

    HttpResponse::NoContent().finish()
}

// favicon.ico处理函数
pub async fn handle_favicon(req: HttpRequest) -> impl Responder {
    let client_ip = get_client_ip(&req);

    debug!("[{}] 请求favicon.ico", client_ip);

    // 直接返回内嵌的SVG内容
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .header("Cache-Control", "public, max-age=2592000") // 缓存一个月
        .body(FAVICON_SVG)
}
