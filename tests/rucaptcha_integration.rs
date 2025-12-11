//! Integration tests for the RuCaptcha provider.
//!
//! These tests require:
//! - `RUCAPTCHA_API_KEY` environment variable
//! - Optional: `PROXY_HOST`, `PROXY_PORT`, `PROXY_TYPE`, `PROXY_USER`, `PROXY_PASSWORD`
//!
//! Run with: `cargo test --all-features -- --ignored rucaptcha`

mod common;

use captcha_solvers::providers::rucaptcha::{
    ProxyConfig, RucaptchaClient, RucaptchaProvider, RucaptchaTask,
};
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceConfig, CaptchaSolverServiceTrait};
use std::time::Duration;

// =============================================================================
// Demo Site Keys from https://2captcha.com/demo
// =============================================================================

/// ReCaptcha V2 demo site key
const RECAPTCHA_V2_SITEKEY: &str = "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd";
const RECAPTCHA_V2_URL: &str = "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high";

/// ReCaptcha V2 Invisible demo site key
const RECAPTCHA_V2_INVISIBLE_SITEKEY: &str = "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd";
const RECAPTCHA_V2_INVISIBLE_URL: &str = "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high";

/// ReCaptcha V2 Enterprise demo site key
const RECAPTCHA_V2_ENTERPRISE_SITEKEY: &str = "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd";
const RECAPTCHA_V2_ENTERPRISE_URL: &str = "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high";

/// ReCaptcha V3 demo site key
const RECAPTCHA_V3_SITEKEY: &str = "6Le0xVgUAAAAAIt20XEB4rVhYOODgTl00d8juDob";
const RECAPTCHA_V3_URL: &str = "https://lessons.zennolab.com/captchas/recaptcha/v3.php?level=beta";
const RECAPTCHA_V3_ACTION: &str = "myverify";
const RECAPTCHA_V3_MIN_SCORE: f32 = 0.9;

/// ReCaptcha V3 Enterprise demo site key
const RECAPTCHA_V3_ENTERPRISE_SITEKEY: &str = "6Le0xVgUAAAAAIt20XEB4rVhYOODgTl00d8juDob";
const RECAPTCHA_V3_ENTERPRISE_URL: &str = "https://lessons.zennolab.com/captchas/recaptcha/v3.php?level=beta";
const RECAPTCHA_V3_ENTERPRISE_MIN_SCORE: f32 = 0.9;

/// Cloudflare Turnstile demo site key
const TURNSTILE_SITEKEY: &str = "0x4AAAAAABhlz7Ei4byodYjs";
const TURNSTILE_URL: &str = "https://visa.vfsglobal.com/uzb/ru/ltu/login";

// =============================================================================
// Helper Functions
// =============================================================================

fn create_service(api_key: String) -> CaptchaSolverService<RucaptchaProvider> {
    let client = RucaptchaClient::new(api_key).expect("Failed to create client");
    let provider = RucaptchaProvider::new(client);
    CaptchaSolverService::new(provider, CaptchaSolverServiceConfig::default())
}

// =============================================================================
// Client Tests
// =============================================================================

/// Test that the client can be created with valid API key
#[tokio::test]
#[ignore]
async fn test_rucaptcha_client_creation() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let client = RucaptchaClient::new(api_key.unwrap());
    assert!(client.is_ok());
}

/// Test that API errors are properly returned for invalid API key
#[tokio::test]
#[ignore]
async fn test_rucaptcha_invalid_api_key() {
    let client = RucaptchaClient::new("invalid_api_key_12345").expect("Failed to create client");
    let provider = RucaptchaProvider::new(client);
    let service = CaptchaSolverService::new(provider, CaptchaSolverServiceConfig::default());

    let task = RucaptchaTask::turnstile("https://example.com", "test_key");

    let result = service.solve_captcha(task, Duration::from_secs(30)).await;
    assert!(result.is_err());
    println!("Got expected error for invalid API key: {:?}", result.err());
}

// =============================================================================
// ReCaptcha V2 Tests
// =============================================================================

/// Test solving ReCaptcha V2 (proxyless)
#[tokio::test]
#[ignore]
async fn test_rucaptcha_recaptcha_v2() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::recaptcha_v2(RECAPTCHA_V2_URL, RECAPTCHA_V2_SITEKEY);

    println!("Solving ReCaptcha V2...");
    let result = service.solve_captcha(task, Duration::from_secs(120)).await;

    match result {
        Ok(solution) => {
            let recaptcha = solution.into_recaptcha();
            let token = recaptcha.token();
            assert!(!token.is_empty());
            println!("Successfully solved ReCaptcha V2");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

/// Test solving ReCaptcha V2 Invisible (proxyless)
#[tokio::test]
#[ignore]
async fn test_rucaptcha_recaptcha_v2_invisible() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::recaptcha_v2_invisible(
        RECAPTCHA_V2_INVISIBLE_URL,
        RECAPTCHA_V2_INVISIBLE_SITEKEY,
    );

    println!("Solving ReCaptcha V2 Invisible...");
    let result = service.solve_captcha(task, Duration::from_secs(120)).await;

    match result {
        Ok(solution) => {
            let recaptcha = solution.into_recaptcha();
            let token = recaptcha.token();
            assert!(!token.is_empty());
            println!("Successfully solved ReCaptcha V2 Invisible");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

/// Test solving ReCaptcha V2 with proxy
#[tokio::test]
#[ignore]
async fn test_rucaptcha_recaptcha_v2_with_proxy() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let proxy = common::proxy_config();
    skip_if_no_proxy!(proxy);

    let proxy = proxy.unwrap();
    println!(
        "Using proxy: {}:{} ({})",
        proxy.address,
        proxy.port,
        proxy.type_str()
    );

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::recaptcha_v2_with_proxy(RECAPTCHA_V2_URL, RECAPTCHA_V2_SITEKEY, proxy);

    println!("Solving ReCaptcha V2 with proxy...");
    let result = service.solve_captcha(task, Duration::from_secs(180)).await;

    match result {
        Ok(solution) => {
            let recaptcha = solution.into_recaptcha();
            let token = recaptcha.token();
            assert!(!token.is_empty());
            println!("Successfully solved ReCaptcha V2 with proxy");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

// =============================================================================
// ReCaptcha V2 Enterprise Tests
// =============================================================================

/// Test solving ReCaptcha V2 Enterprise (proxyless)
#[tokio::test]
#[ignore]
async fn test_rucaptcha_recaptcha_v2_enterprise() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::recaptcha_v2_enterprise(
        RECAPTCHA_V2_ENTERPRISE_URL,
        RECAPTCHA_V2_ENTERPRISE_SITEKEY,
    );

    println!("Solving ReCaptcha V2 Enterprise...");
    let result = service.solve_captcha(task, Duration::from_secs(120)).await;

    match result {
        Ok(solution) => {
            let recaptcha = solution.into_recaptcha();
            let token = recaptcha.token();
            assert!(!token.is_empty());
            println!("Successfully solved ReCaptcha V2 Enterprise");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

/// Test solving ReCaptcha V2 Enterprise with proxy
#[tokio::test]
#[ignore]
async fn test_rucaptcha_recaptcha_v2_enterprise_with_proxy() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let proxy = common::proxy_config();
    skip_if_no_proxy!(proxy);

    let proxy = proxy.unwrap();
    println!(
        "Using proxy: {}:{} ({})",
        proxy.address,
        proxy.port,
        proxy.type_str()
    );

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::recaptcha_v2_enterprise_with_proxy(
        RECAPTCHA_V2_ENTERPRISE_URL,
        RECAPTCHA_V2_ENTERPRISE_SITEKEY,
        proxy,
    );

    println!("Solving ReCaptcha V2 Enterprise with proxy...");
    let result = service.solve_captcha(task, Duration::from_secs(180)).await;

    match result {
        Ok(solution) => {
            let recaptcha = solution.into_recaptcha();
            let token = recaptcha.token();
            assert!(!token.is_empty());
            println!("Successfully solved ReCaptcha V2 Enterprise with proxy");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

// =============================================================================
// ReCaptcha V3 Tests
// =============================================================================

/// Test solving ReCaptcha V3 (proxyless)
#[tokio::test]
#[ignore]
async fn test_rucaptcha_recaptcha_v3() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::recaptcha_v3(RECAPTCHA_V3_URL, RECAPTCHA_V3_SITEKEY, RECAPTCHA_V3_MIN_SCORE);

    println!("Solving ReCaptcha V3...");
    let result = service.solve_captcha(task, Duration::from_secs(120)).await;

    match result {
        Ok(solution) => {
            let recaptcha = solution.into_recaptcha();
            let token = recaptcha.token();
            assert!(!token.is_empty());
            println!("Successfully solved ReCaptcha V3");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

/// Test solving ReCaptcha V3 with action (proxyless)
#[tokio::test]
#[ignore]
async fn test_rucaptcha_recaptcha_v3_with_action() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::recaptcha_v3_with_action(
        RECAPTCHA_V3_URL,
        RECAPTCHA_V3_SITEKEY,
        RECAPTCHA_V3_MIN_SCORE,
        RECAPTCHA_V3_ACTION,
    );

    println!("Solving ReCaptcha V3 with action '{}'...", RECAPTCHA_V3_ACTION);
    let result = service.solve_captcha(task, Duration::from_secs(120)).await;

    match result {
        Ok(solution) => {
            let recaptcha = solution.into_recaptcha();
            let token = recaptcha.token();
            assert!(!token.is_empty());
            println!("Successfully solved ReCaptcha V3 with action");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

/// Test solving ReCaptcha V3 Enterprise (proxyless)
#[tokio::test]
#[ignore]
async fn test_rucaptcha_recaptcha_v3_enterprise() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::recaptcha_v3_enterprise(
        RECAPTCHA_V3_ENTERPRISE_URL,
        RECAPTCHA_V3_ENTERPRISE_SITEKEY,
        RECAPTCHA_V3_ENTERPRISE_MIN_SCORE,
    );

    println!("Solving ReCaptcha V3 Enterprise...");
    let result = service.solve_captcha(task, Duration::from_secs(120)).await;

    match result {
        Ok(solution) => {
            let recaptcha = solution.into_recaptcha();
            let token = recaptcha.token();
            assert!(!token.is_empty());
            println!("Successfully solved ReCaptcha V3 Enterprise");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

// =============================================================================
// Turnstile Tests
// =============================================================================

/// Test solving Cloudflare Turnstile (proxyless)
#[tokio::test]
#[ignore]
async fn test_rucaptcha_turnstile() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::turnstile(TURNSTILE_URL, TURNSTILE_SITEKEY);

    println!("Solving Cloudflare Turnstile...");
    let result = service.solve_captcha(task, Duration::from_secs(120)).await;

    match result {
        Ok(solution) => {
            let turnstile = solution.into_turnstile();
            let token = turnstile.token();
            assert!(!token.is_empty());
            println!("Successfully solved Cloudflare Turnstile");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

/// Test solving Cloudflare Turnstile with proxy
#[tokio::test]
#[ignore]
async fn test_rucaptcha_turnstile_with_proxy() {
    let api_key = common::rucaptcha_api_key();
    skip_if_no_api_key!(api_key);

    let proxy = common::proxy_config();
    skip_if_no_proxy!(proxy);

    let proxy = proxy.unwrap();
    println!(
        "Using proxy: {}:{} ({})",
        proxy.address,
        proxy.port,
        proxy.type_str()
    );

    let service = create_service(api_key.unwrap());
    let task = RucaptchaTask::turnstile_with_proxy(TURNSTILE_URL, TURNSTILE_SITEKEY, proxy);

    println!("Solving Cloudflare Turnstile with proxy...");
    let result = service.solve_captcha(task, Duration::from_secs(180)).await;

    match result {
        Ok(solution) => {
            let turnstile = solution.into_turnstile();
            let token = turnstile.token();
            assert!(!token.is_empty());
            println!("Successfully solved Cloudflare Turnstile with proxy");
            println!("Token length: {}", token.len());
        }
        Err(e) => {
            panic!("Failed to solve captcha: {}", e);
        }
    }
}

// =============================================================================
// Serialization Tests (non-ignored, no API key required)
// =============================================================================

/// Test ReCaptcha V2 task serialization
#[test]
fn test_recaptcha_v2_serialization() {
    let task = RucaptchaTask::recaptcha_v2(RECAPTCHA_V2_URL, RECAPTCHA_V2_SITEKEY);
    let json = serde_json::to_string_pretty(&task).unwrap();
    println!("ReCaptcha V2 Task JSON:\n{}", json);

    assert!(json.contains("RecaptchaV2TaskProxyless"));
    assert!(json.contains(RECAPTCHA_V2_SITEKEY));
    assert!(json.contains(RECAPTCHA_V2_URL));
}

/// Test ReCaptcha V2 with proxy task serialization
#[test]
fn test_recaptcha_v2_with_proxy_serialization() {
    let proxy = ProxyConfig::http("192.168.1.1", 8080).with_auth("user", "pass");
    let task = RucaptchaTask::recaptcha_v2_with_proxy(RECAPTCHA_V2_URL, RECAPTCHA_V2_SITEKEY, proxy);
    let json = serde_json::to_string_pretty(&task).unwrap();
    println!("ReCaptcha V2 with Proxy Task JSON:\n{}", json);

    assert!(json.contains("RecaptchaV2Task"));
    assert!(!json.contains("Proxyless")); // RecaptchaV2Task, not RecaptchaV2TaskProxyless
    assert!(json.contains("proxyType"));
    assert!(json.contains("proxyAddress"));
    assert!(json.contains("proxyPort"));
}

/// Test ReCaptcha V3 task serialization
#[test]
fn test_recaptcha_v3_serialization() {
    let task = RucaptchaTask::recaptcha_v3(RECAPTCHA_V3_URL, RECAPTCHA_V3_SITEKEY, RECAPTCHA_V3_MIN_SCORE);
    let json = serde_json::to_string_pretty(&task).unwrap();
    println!("ReCaptcha V3 Task JSON:\n{}", json);

    assert!(json.contains("RecaptchaV3TaskProxyless"));
    assert!(json.contains(RECAPTCHA_V3_SITEKEY));
    assert!(json.contains("minScore"));
    assert!(json.contains("0.9"));
}

/// Test ReCaptcha V3 with action task serialization
#[test]
fn test_recaptcha_v3_with_action_serialization() {
    let task = RucaptchaTask::recaptcha_v3_with_action(
        RECAPTCHA_V3_URL,
        RECAPTCHA_V3_SITEKEY,
        RECAPTCHA_V3_MIN_SCORE,
        RECAPTCHA_V3_ACTION,
    );
    let json = serde_json::to_string_pretty(&task).unwrap();
    println!("ReCaptcha V3 with Action Task JSON:\n{}", json);

    assert!(json.contains("RecaptchaV3TaskProxyless"));
    assert!(json.contains(RECAPTCHA_V3_SITEKEY));
    assert!(json.contains("pageAction"));
    assert!(json.contains(RECAPTCHA_V3_ACTION));
}

/// Test proxy configuration serialization
#[test]
fn test_proxy_config_serialization() {
    let proxy = ProxyConfig::socks5("192.168.1.1", 1080).with_auth("user", "pass");
    let task = RucaptchaTask::recaptcha_v2_with_proxy("https://example.com", "site_key", proxy);

    let json = serde_json::to_string_pretty(&task).unwrap();
    println!("Task with Proxy JSON:\n{}", json);

    // Note: pretty print adds spaces and newlines
    assert!(json.contains("socks5"));
    assert!(json.contains("192.168.1.1"));
    assert!(json.contains("1080"));
    assert!(json.contains("\"proxyLogin\": \"user\""));
    assert!(json.contains("\"proxyPassword\": \"pass\""));
}

/// Test Turnstile task serialization
#[test]
fn test_turnstile_serialization() {
    let task = RucaptchaTask::turnstile(TURNSTILE_URL, TURNSTILE_SITEKEY);
    let json = serde_json::to_string_pretty(&task).unwrap();
    println!("Turnstile Task JSON:\n{}", json);

    assert!(json.contains("TurnstileTaskProxyless"));
    assert!(json.contains(TURNSTILE_SITEKEY));
}

/// Test Turnstile with proxy task serialization
#[test]
fn test_turnstile_with_proxy_serialization() {
    let proxy = ProxyConfig::http("proxy.example.com", 8080);
    let task = RucaptchaTask::turnstile_with_proxy(TURNSTILE_URL, TURNSTILE_SITEKEY, proxy);
    let json = serde_json::to_string_pretty(&task).unwrap();
    println!("Turnstile with Proxy Task JSON:\n{}", json);

    assert!(json.contains("TurnstileTask"));
    assert!(!json.contains("Proxyless")); // TurnstileTask, not TurnstileTaskProxyless
    assert!(json.contains("proxyType"));
    assert!(json.contains("proxyAddress"));
}
