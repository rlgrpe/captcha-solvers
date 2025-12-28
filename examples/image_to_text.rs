//! Image to text captcha solving example.
//!
//! Run with: `cargo run --example image_to_text`
//!
//! Required environment variable:
//! - `CAPSOLVER_API_KEY` - Your Capsolver API key
//!
//! This example demonstrates how to solve image captchas using OCR recognition.
//! The image can be provided as raw bytes or a pre-encoded base64 string.

use captcha_solvers::capsolver::CapsolverProvider;
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ImageToText};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment
    let api_key = env::var("CAPSOLVER_API_KEY").expect("CAPSOLVER_API_KEY must be set");

    // Create provider and service
    let provider = CapsolverProvider::new(api_key)?;
    let service = CaptchaSolverService::new(provider);

    // Example 1: From base64 string
    // This is a simple test image (1x1 pixel PNG - for demonstration only)
    // In real usage, you would use an actual captcha image
    let base64_image = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

    let task = ImageToText::from_base64(base64_image)
        .with_module("module_005") // Use "module_005" OCR module (Capsolver)
        .with_website_url("https://example.com"); // Optional: helps improve accuracy

    println!("Solving image captcha...");

    match service.solve_captcha(task).await {
        Ok(solution) => {
            let image_solution = solution.into_image_to_text();
            println!("Recognized text: {}", image_solution.text());
        }
        Err(e) => {
            eprintln!("Error solving captcha: {}", e);
        }
    }

    // Example 2: From raw bytes (e.g., reading from a file)
    // Uncomment to use:
    //
    // let image_bytes = std::fs::read("captcha.png")?;
    // let task = ImageToText::from_bytes(image_bytes)
    //     .case_sensitive()      // Answer is case-sensitive
    //     .numbers_only();       // Answer contains only numbers
    //
    // let solution = service.solve_captcha(task).await?;
    // println!("Recognized text: {}", solution.into_image_to_text().text());

    // Example 3: With RuCaptcha-specific options
    // Uncomment to use:
    //
    // let task = ImageToText::from_base64(base64_image)
    //     .case_sensitive()
    //     .with_min_length(4)
    //     .with_max_length(8)
    //     .with_comment("Enter red text only");

    Ok(())
}
