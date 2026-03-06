//! Integration tests for the CapMonster provider.
//!
//! These tests require:
//! - `CAPMONSTER_API_KEY` environment variable
//! - Optional challenge/waitroom variables for advanced scenarios
//! - Optional proxy env (`PROXY_HOST`, `PROXY_PORT`, ...)
//!
//! Run with: `cargo test --all-features -- --ignored capmonster`

mod common;

use captcha_solvers::capmonster::CapmonsterProvider;
use captcha_solvers::{
    CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2, ReCaptchaV3, Turnstile,
    TurnstileChallenge, TurnstileWaitRoom,
};

/// ReCaptcha V2 demo site key
const RECAPTCHA_V2_SITEKEY: &str = "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd";
const RECAPTCHA_V2_URL: &str =
    "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high";

/// ReCaptcha V3 demo site key
const RECAPTCHA_V3_SITEKEY: &str = "6Le0xVgUAAAAAIt20XEB4rVhYOODgTl00d8juDob";
const RECAPTCHA_V3_URL: &str = "https://lessons.zennolab.com/captchas/recaptcha/v3.php?level=beta";

/// Cloudflare Turnstile demo site key
const TURNSTILE_SITEKEY: &str = "0x4AAAAAABhlz7Ei4byodYjs";
const TURNSTILE_URL: &str = "https://visa.vfsglobal.com/uzb/ru/ltu/login";

fn create_service(api_key: String) -> CaptchaSolverService<CapmonsterProvider> {
    let provider = CapmonsterProvider::new(api_key).expect("Failed to create provider");
    CaptchaSolverService::new(provider)
}

fn env_var(name: &str) -> Option<String> {
    common::load_env();
    std::env::var(name).ok()
}

#[tokio::test]
#[ignore]
async fn test_capmonster_provider_creation() {
    let api_key = common::capmonster_api_key();
    skip_if_no_api_key!(api_key);

    let provider = CapmonsterProvider::new(api_key.unwrap());
    assert!(provider.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_capmonster_recaptcha_v2() {
    let api_key = common::capmonster_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = ReCaptchaV2::new(RECAPTCHA_V2_URL, RECAPTCHA_V2_SITEKEY);

    let result = service.solve_captcha(task).await;
    match result {
        Ok(solution) => {
            let token = solution.into_recaptcha().token().to_string();
            assert!(!token.is_empty());
        }
        Err(e) => panic!("Failed to solve ReCaptcha V2: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_capmonster_recaptcha_v2_enterprise() {
    let api_key = common::capmonster_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = ReCaptchaV2::new(RECAPTCHA_V2_URL, RECAPTCHA_V2_SITEKEY).enterprise();

    let result = service.solve_captcha(task).await;
    match result {
        Ok(solution) => assert!(!solution.into_recaptcha().token().is_empty()),
        Err(e) => panic!("Failed to solve ReCaptcha V2 Enterprise: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_capmonster_recaptcha_v3() {
    let api_key = common::capmonster_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = ReCaptchaV3::new(RECAPTCHA_V3_URL, RECAPTCHA_V3_SITEKEY)
        .with_action("myverify")
        .with_min_score(0.7);

    let result = service.solve_captcha(task).await;
    match result {
        Ok(solution) => assert!(!solution.into_recaptcha().token().is_empty()),
        Err(e) => panic!("Failed to solve ReCaptcha V3: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_capmonster_recaptcha_v3_enterprise() {
    let api_key = common::capmonster_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = ReCaptchaV3::new(RECAPTCHA_V3_URL, RECAPTCHA_V3_SITEKEY)
        .enterprise()
        .with_min_score(0.7);

    let result = service.solve_captcha(task).await;
    match result {
        Ok(solution) => assert!(!solution.into_recaptcha().token().is_empty()),
        Err(e) => panic!("Failed to solve ReCaptcha V3 Enterprise: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_capmonster_turnstile() {
    let api_key = common::capmonster_api_key();
    skip_if_no_api_key!(api_key);

    let service = create_service(api_key.unwrap());
    let task = Turnstile::new(TURNSTILE_URL, TURNSTILE_SITEKEY);

    let result = service.solve_captcha(task).await;
    match result {
        Ok(solution) => {
            let turnstile = solution.into_turnstile();
            assert!(!turnstile.token().unwrap().is_empty());
        }
        Err(e) => panic!("Failed to solve Turnstile: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_capmonster_turnstile_challenge_token() {
    let api_key = common::capmonster_api_key();
    skip_if_no_api_key!(api_key);

    let (url, sitekey, action, data, pagedata, user_agent) = match (
        env_var("CAPMONSTER_CHALLENGE_URL"),
        env_var("CAPMONSTER_CHALLENGE_SITEKEY"),
        env_var("CAPMONSTER_CHALLENGE_ACTION"),
        env_var("CAPMONSTER_CHALLENGE_DATA"),
        env_var("CAPMONSTER_CHALLENGE_PAGEDATA"),
        env_var("CAPMONSTER_CHALLENGE_USER_AGENT"),
    ) {
        (Some(url), Some(sitekey), Some(action), Some(data), Some(pagedata), Some(user_agent)) => {
            (url, sitekey, action, data, pagedata, user_agent)
        }
        _ => {
            eprintln!("Skipping test: challenge token params are not set");
            return;
        }
    };

    let service = create_service(api_key.unwrap());
    let task = TurnstileChallenge::token(url, sitekey, action, data, pagedata, user_agent);

    let result = service.solve_captcha(task).await;
    match result {
        Ok(solution) => {
            let turnstile = solution.into_turnstile();
            assert!(turnstile.token().is_some());
        }
        Err(e) => panic!("Failed to solve Turnstile Challenge token mode: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_capmonster_turnstile_challenge_cf_clearance() {
    let api_key = common::capmonster_api_key();
    skip_if_no_api_key!(api_key);

    let (url, sitekey, html_page_base64, user_agent) = match (
        env_var("CAPMONSTER_CHALLENGE_URL"),
        env_var("CAPMONSTER_CHALLENGE_SITEKEY"),
        env_var("CAPMONSTER_CHALLENGE_HTML_BASE64"),
        env_var("CAPMONSTER_CHALLENGE_USER_AGENT"),
    ) {
        (Some(url), Some(sitekey), Some(html_page_base64), Some(user_agent)) => {
            (url, sitekey, html_page_base64, user_agent)
        }
        _ => {
            eprintln!("Skipping test: challenge cf_clearance params are not set");
            return;
        }
    };

    let proxy = common::proxy_config();
    skip_if_no_proxy!(proxy);

    let service = create_service(api_key.unwrap());
    let task = TurnstileChallenge::cf_clearance(
        url,
        sitekey,
        html_page_base64,
        user_agent,
        proxy.unwrap(),
    );

    let result = service.solve_captcha(task).await;
    match result {
        Ok(solution) => {
            let turnstile = solution.into_turnstile();
            assert!(turnstile.cf_clearance().is_some());
        }
        Err(e) => panic!(
            "Failed to solve Turnstile Challenge cf_clearance mode: {}",
            e
        ),
    }
}

#[tokio::test]
#[ignore]
async fn test_capmonster_turnstile_waitroom() {
    let api_key = common::capmonster_api_key();
    skip_if_no_api_key!(api_key);

    let (url, sitekey, html_page_base64, user_agent) = match (
        env_var("CAPMONSTER_WAITROOM_URL"),
        env_var("CAPMONSTER_WAITROOM_SITEKEY"),
        env_var("CAPMONSTER_WAITROOM_HTML_BASE64"),
        env_var("CAPMONSTER_WAITROOM_USER_AGENT"),
    ) {
        (Some(url), Some(sitekey), Some(html_page_base64), Some(user_agent)) => {
            (url, sitekey, html_page_base64, user_agent)
        }
        _ => {
            eprintln!("Skipping test: wait_room params are not set");
            return;
        }
    };

    let proxy = common::proxy_config();
    skip_if_no_proxy!(proxy);

    let service = create_service(api_key.unwrap());
    let task = TurnstileWaitRoom::new(url, sitekey, html_page_base64, user_agent, proxy.unwrap());

    let result = service.solve_captcha(task).await;
    match result {
        Ok(solution) => {
            let turnstile = solution.into_turnstile();
            assert!(turnstile.cf_clearance().is_some());
        }
        Err(e) => panic!("Failed to solve Turnstile wait_room mode: {}", e),
    }
}

#[test]
fn test_new_task_types_builders() {
    let token_task = TurnstileChallenge::token(
        "https://example.com",
        "key",
        "managed",
        "data",
        "page-data",
        "Mozilla/5.0",
    );
    assert!(token_task.is_token_mode());

    let proxy = captcha_solvers::ProxyConfig::http("127.0.0.1", 8080);
    let clearance_task = TurnstileChallenge::cf_clearance(
        "https://example.com",
        "key",
        "base64",
        "Mozilla/5.0",
        proxy.clone(),
    );
    assert!(clearance_task.is_cf_clearance_mode());

    let waitroom =
        TurnstileWaitRoom::new("https://example.com", "key", "base64", "Mozilla/5.0", proxy);
    assert_eq!(waitroom.website_key, "key");
}
