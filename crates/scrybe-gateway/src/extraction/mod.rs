//! Server-side signal extraction from HTTP requests.

pub mod headers;
pub mod ip;

pub use headers::{extract_headers, extract_http_version};
pub use ip::extract_ip_info;
