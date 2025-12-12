//! Proxy configuration for captcha solving tasks.
//!
//! This module provides a unified proxy configuration that can be used
//! with any provider that supports proxy-based captcha solving.

use serde::{Deserialize, Serialize, Serializer};

/// Proxy type for tasks requiring custom proxy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
}

impl ProxyType {
    /// Get the string representation for Capsolver API (includes https)
    pub fn as_capsolver_str(&self) -> &'static str {
        match self {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks4 => "socks4",
            ProxyType::Socks5 => "socks5",
        }
    }

    /// Get the string representation for RuCaptcha API (http/https both map to http)
    pub fn as_rucaptcha_str(&self) -> &'static str {
        match self {
            ProxyType::Http | ProxyType::Https => "http",
            ProxyType::Socks4 => "socks4",
            ProxyType::Socks5 => "socks5",
        }
    }
}

/// Proxy fields for serialization into task payloads (Capsolver format)
///
/// This struct can be flattened into task variants to avoid repeating
/// the same 5 proxy fields in every variant.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapsolverProxyFields {
    #[serde(rename = "proxyType", serialize_with = "serialize_capsolver_proxy_type")]
    pub proxy_type: ProxyType,
    #[serde(rename = "proxyAddress")]
    pub proxy_address: String,
    #[serde(rename = "proxyPort")]
    pub proxy_port: u16,
    #[serde(rename = "proxyLogin", skip_serializing_if = "Option::is_none")]
    pub proxy_login: Option<String>,
    #[serde(rename = "proxyPassword", skip_serializing_if = "Option::is_none")]
    pub proxy_password: Option<String>,
}

/// Proxy fields for serialization into task payloads (RuCaptcha format)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RucaptchaProxyFields {
    #[serde(rename = "proxyType", serialize_with = "serialize_rucaptcha_proxy_type")]
    pub proxy_type: ProxyType,
    #[serde(rename = "proxyAddress")]
    pub proxy_address: String,
    #[serde(rename = "proxyPort")]
    pub proxy_port: u16,
    #[serde(rename = "proxyLogin", skip_serializing_if = "Option::is_none")]
    pub proxy_login: Option<String>,
    #[serde(rename = "proxyPassword", skip_serializing_if = "Option::is_none")]
    pub proxy_password: Option<String>,
}

/// Serialize ProxyType for Capsolver API
pub fn serialize_capsolver_proxy_type<S>(proxy_type: &ProxyType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(proxy_type.as_capsolver_str())
}

/// Serialize ProxyType for RuCaptcha API
pub fn serialize_rucaptcha_proxy_type<S>(proxy_type: &ProxyType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(proxy_type.as_rucaptcha_str())
}

impl From<ProxyConfig> for CapsolverProxyFields {
    fn from(config: ProxyConfig) -> Self {
        Self {
            proxy_type: config.proxy_type,
            proxy_address: config.address,
            proxy_port: config.port,
            proxy_login: config.login,
            proxy_password: config.password,
        }
    }
}

impl From<ProxyConfig> for RucaptchaProxyFields {
    fn from(config: ProxyConfig) -> Self {
        Self {
            proxy_type: config.proxy_type,
            proxy_address: config.address,
            proxy_port: config.port,
            proxy_login: config.login,
            proxy_password: config.password,
        }
    }
}

/// Proxy configuration for captcha solving tasks
///
/// # Examples
///
/// ```rust
/// use captcha_solvers::ProxyConfig;
///
/// // HTTP proxy without auth
/// let proxy = ProxyConfig::http("192.168.1.1", 8080);
///
/// // SOCKS5 proxy with auth
/// let proxy = ProxyConfig::socks5("proxy.example.com", 1080)
///     .with_auth("user", "pass");
///
/// // Convert to Capsolver string format
/// let proxy_str = proxy.to_string_format();
/// // Result: "socks5:proxy.example.com:1080:user:pass"
/// ```
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub address: String,
    pub port: u16,
    pub login: Option<String>,
    pub password: Option<String>,
}

impl ProxyConfig {
    /// Create a new HTTP proxy configuration
    pub fn http(address: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Http,
            address: address.into(),
            port,
            login: None,
            password: None,
        }
    }

    /// Create a new HTTPS proxy configuration
    pub fn https(address: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Https,
            address: address.into(),
            port,
            login: None,
            password: None,
        }
    }

    /// Create a new SOCKS4 proxy configuration
    pub fn socks4(address: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Socks4,
            address: address.into(),
            port,
            login: None,
            password: None,
        }
    }

    /// Create a new SOCKS5 proxy configuration
    pub fn socks5(address: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Socks5,
            address: address.into(),
            port,
            login: None,
            password: None,
        }
    }

    /// Add authentication credentials
    pub fn with_auth(mut self, login: impl Into<String>, password: impl Into<String>) -> Self {
        self.login = Some(login.into());
        self.password = Some(password.into());
        self
    }

    /// Convert to string format: `type:address:port[:user:pass]`
    ///
    /// This format is used by Capsolver and similar services.
    pub fn to_string_format(&self) -> String {
        let type_str = match self.proxy_type {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks4 => "socks4",
            ProxyType::Socks5 => "socks5",
        };

        match (&self.login, &self.password) {
            (Some(login), Some(password)) => {
                format!(
                    "{}:{}:{}:{}:{}",
                    type_str, self.address, self.port, login, password
                )
            }
            _ => {
                format!("{}:{}:{}", type_str, self.address, self.port)
            }
        }
    }

    /// Get the proxy type string for RuCaptcha-style APIs
    pub fn type_str(&self) -> &'static str {
        match self.proxy_type {
            ProxyType::Http | ProxyType::Https => "http",
            ProxyType::Socks4 => "socks4",
            ProxyType::Socks5 => "socks5",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_proxy() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        assert_eq!(proxy.proxy_type, ProxyType::Http);
        assert_eq!(proxy.address, "192.168.1.1");
        assert_eq!(proxy.port, 8080);
        assert!(proxy.login.is_none());
        assert!(proxy.password.is_none());
    }

    #[test]
    fn test_socks5_proxy_with_auth() {
        let proxy = ProxyConfig::socks5("proxy.example.com", 1080).with_auth("user", "pass");
        assert_eq!(proxy.proxy_type, ProxyType::Socks5);
        assert_eq!(proxy.address, "proxy.example.com");
        assert_eq!(proxy.port, 1080);
        assert_eq!(proxy.login.as_deref(), Some("user"));
        assert_eq!(proxy.password.as_deref(), Some("pass"));
    }

    #[test]
    fn test_to_string_format_without_auth() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        assert_eq!(proxy.to_string_format(), "http:192.168.1.1:8080");
    }

    #[test]
    fn test_to_string_format_with_auth() {
        let proxy = ProxyConfig::socks5("proxy.example.com", 1080).with_auth("user", "pass");
        assert_eq!(
            proxy.to_string_format(),
            "socks5:proxy.example.com:1080:user:pass"
        );
    }

    #[test]
    fn test_type_str() {
        assert_eq!(ProxyConfig::http("a", 1).type_str(), "http");
        assert_eq!(ProxyConfig::https("a", 1).type_str(), "http");
        assert_eq!(ProxyConfig::socks4("a", 1).type_str(), "socks4");
        assert_eq!(ProxyConfig::socks5("a", 1).type_str(), "socks5");
    }
}