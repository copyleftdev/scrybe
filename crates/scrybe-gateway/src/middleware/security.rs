//! Security headers middleware.

use axum::{
    extract::Request,
    http::header::{self, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Add security headers to all responses.
///
/// Headers added:
/// - `Strict-Transport-Security`: HSTS with 1-year max-age
/// - `X-Content-Type-Options`: Prevent MIME sniffing
/// - `X-Frame-Options`: Prevent clickjacking
/// - `Content-Security-Policy`: Strict CSP policy
/// - `X-XSS-Protection`: Enable XSS protection (legacy browsers)
/// - `Referrer-Policy`: Control referrer information
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // HSTS - Force HTTPS for 1 year
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // Prevent MIME sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // Content Security Policy - very strict for API
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );

    // XSS Protection (legacy browsers)
    headers.insert(
        header::X_XSS_PROTECTION,
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer Policy
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );

    response
}

#[cfg(test)]
mod tests {
    // TODO: Add integration tests for security headers
    // Unit testing middleware requires mocking Next, which is not straightforward
    // Integration tests will verify headers are properly added
}
