//! Middleware for authentication, rate limiting, and security.
//!
//! These middleware components are ready for use but not yet fully
//! integrated pending complete testing and Redis setup.

pub mod auth;
pub mod rate_limit;
pub mod security;

// Ready for integration (allow unused until wired up)
#[allow(unused_imports)]
pub use auth::hmac_auth;
#[allow(unused_imports)]
pub use rate_limit::rate_limit_layer;
pub use security::security_headers;
