//! Common helpers for integration tests.

use std::env;

/// Load environment variables from .env file
pub fn load_env() {
    let _ = dotenvy::dotenv();
}

/// Get Capsolver API key from environment
pub fn capsolver_api_key() -> Option<String> {
    load_env();
    env::var("CAPSOLVER_API_KEY").ok()
}

/// Get RuCaptcha API key from environment
pub fn rucaptcha_api_key() -> Option<String> {
    load_env();
    env::var("RUCAPTCHA_API_KEY").ok()
}

/// Get proxy configuration from environment
///
/// Expected environment variables:
/// - `PROXY_HOST` - Proxy hostname or IP
/// - `PROXY_PORT` - Proxy port number
/// - `PROXY_TYPE` - One of: http, https, socks4, socks5 (default: http)
/// - `PROXY_USER` - Optional proxy username
/// - `PROXY_PASSWORD` - Optional proxy password
pub fn proxy_config() -> Option<captcha_solvers::ProxyConfig> {
    load_env();

    let host = env::var("PROXY_HOST").ok()?;
    let port: u16 = env::var("PROXY_PORT").ok()?.parse().ok()?;
    let proxy_type = env::var("PROXY_TYPE").ok().unwrap_or_else(|| "http".to_string());

    let proxy = match proxy_type.to_lowercase().as_str() {
        "https" => captcha_solvers::ProxyConfig::https(&host, port),
        "socks4" => captcha_solvers::ProxyConfig::socks4(&host, port),
        "socks5" => captcha_solvers::ProxyConfig::socks5(&host, port),
        _ => captcha_solvers::ProxyConfig::http(&host, port),
    };

    // Add auth if provided
    let proxy = match (env::var("PROXY_USER").ok(), env::var("PROXY_PASSWORD").ok()) {
        (Some(user), Some(pass)) => proxy.with_auth(user, pass),
        _ => proxy,
    };

    Some(proxy)
}

/// Skip test if API key is not set
#[macro_export]
macro_rules! skip_if_no_api_key {
    ($key:expr) => {
        if $key.is_none() {
            eprintln!("Skipping test: API key not set");
            return;
        }
    };
}

/// Skip test if proxy is not set
#[macro_export]
macro_rules! skip_if_no_proxy {
    ($proxy:expr) => {
        if $proxy.is_none() {
            eprintln!("Skipping test: Proxy not configured");
            return;
        }
    };
}