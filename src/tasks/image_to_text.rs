//! Image to text captcha task type with builder pattern.
//!
//! This module provides provider-agnostic image to text captcha task definitions
//! that can be converted to any supported provider's format using the `Into` trait.

use base64::{Engine, engine::general_purpose::STANDARD};

/// Image to text captcha task with fluent builder pattern.
///
/// Use this type to create image captcha solving requests that work with any provider.
/// The task can be converted to provider-specific formats using `.into()`.
///
/// # Examples
///
/// ```
/// use captcha_solvers::ImageToText;
///
/// // From raw bytes (automatically base64-encoded)
/// let image_bytes: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
/// let task = ImageToText::from_bytes(image_bytes);
///
/// // From pre-encoded base64 string
/// let task = ImageToText::from_base64("iVBORw0KGgoAAAANSUhEUgAA...");
///
/// // With additional options
/// let task = ImageToText::from_base64("iVBORw0KGgoAAAANSUhEUgAA...")
///     .case_sensitive()
///     .numbers_only()
///     .with_min_length(4)
///     .with_max_length(6);
///
/// // With module (Capsolver-specific)
/// let task = ImageToText::from_base64("iVBORw0KGgoAAAANSUhEUgAA...")
///     .with_module("common");
/// ```
#[derive(Debug, Clone)]
pub struct ImageToText {
    /// Base64 encoded image content (without data URI prefix)
    pub body: String,

    /// Page source URL to improve accuracy (optional)
    pub website_url: Option<String>,

    /// Recognition module to use (Capsolver: "common", "number", etc.)
    pub module: Option<String>,

    /// Whether the answer must contain multiple words separated by space
    pub phrase: bool,

    /// Whether the answer is case-sensitive
    pub case_sensitive: bool,

    /// Numeric constraint:
    /// - 0: no requirements
    /// - 1: numbers only
    /// - 2: letters only
    /// - 3: either numbers or letters
    /// - 4: both numbers AND letters required
    pub numeric: u8,

    /// Whether the captcha requires a math calculation
    pub math: bool,

    /// Minimum answer length (0 = no restriction)
    pub min_length: u32,

    /// Maximum answer length (0 = no restriction)
    pub max_length: u32,

    /// Additional instruction text for workers
    pub comment: Option<String>,

    /// Base64-encoded instruction image for workers
    pub img_instructions: Option<String>,
}

impl ImageToText {
    /// Create a new image to text captcha task from raw image bytes.
    ///
    /// The bytes will be automatically encoded to base64.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw image bytes (PNG, JPEG, GIF, etc.)
    ///
    /// # Example
    ///
    /// ```
    /// use captcha_solvers::ImageToText;
    ///
    /// let image_bytes = std::fs::read("captcha.png").unwrap_or_default();
    /// let task = ImageToText::from_bytes(image_bytes);
    /// ```
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
        Self {
            body: STANDARD.encode(bytes.as_ref()),
            website_url: None,
            module: None,
            phrase: false,
            case_sensitive: false,
            numeric: 0,
            math: false,
            min_length: 0,
            max_length: 0,
            comment: None,
            img_instructions: None,
        }
    }

    /// Create a new image to text captcha task from a pre-encoded base64 string.
    ///
    /// Use this when you already have the image encoded as base64.
    /// The string should NOT include the data URI prefix (e.g., "data:image/png;base64,").
    ///
    /// # Arguments
    ///
    /// * `base64` - Base64 encoded image content
    ///
    /// # Example
    ///
    /// ```
    /// use captcha_solvers::ImageToText;
    ///
    /// let task = ImageToText::from_base64("iVBORw0KGgoAAAANSUhEUgAA...");
    /// ```
    pub fn from_base64(base64: impl Into<String>) -> Self {
        Self {
            body: base64.into(),
            website_url: None,
            module: None,
            phrase: false,
            case_sensitive: false,
            numeric: 0,
            math: false,
            min_length: 0,
            max_length: 0,
            comment: None,
            img_instructions: None,
        }
    }

    /// Set the website URL for improved accuracy.
    pub fn with_website_url(mut self, url: impl Into<String>) -> Self {
        self.website_url = Some(url.into());
        self
    }

    /// Set the recognition module (Capsolver: "common", "number").
    ///
    /// # Capsolver Modules
    ///
    /// - `common` - Default OCR model for general images
    /// - `number` - Optimized for images containing only numbers
    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }

    /// Require the answer to contain multiple space-separated words.
    pub fn phrase(mut self) -> Self {
        self.phrase = true;
        self
    }

    /// Mark the answer as case-sensitive.
    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }

    /// Restrict answer to numbers only.
    pub fn numbers_only(mut self) -> Self {
        self.numeric = 1;
        self
    }

    /// Restrict answer to letters only.
    pub fn letters_only(mut self) -> Self {
        self.numeric = 2;
        self
    }

    /// Allow either numbers or letters (not mixed).
    pub fn numbers_or_letters(mut self) -> Self {
        self.numeric = 3;
        self
    }

    /// Require both numbers and letters.
    pub fn alphanumeric(mut self) -> Self {
        self.numeric = 4;
        self
    }

    /// Set numeric constraint directly.
    ///
    /// - 0: no requirements
    /// - 1: numbers only
    /// - 2: letters only
    /// - 3: either numbers or letters
    /// - 4: both numbers AND letters required
    pub fn with_numeric(mut self, numeric: u8) -> Self {
        self.numeric = numeric;
        self
    }

    /// Mark the captcha as requiring math calculation.
    pub fn math(mut self) -> Self {
        self.math = true;
        self
    }

    /// Set minimum answer length.
    pub fn with_min_length(mut self, length: u32) -> Self {
        self.min_length = length;
        self
    }

    /// Set maximum answer length.
    pub fn with_max_length(mut self, length: u32) -> Self {
        self.max_length = length;
        self
    }

    /// Set additional instruction text for workers.
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Set base64-encoded instruction image for workers.
    pub fn with_img_instructions(mut self, img: impl Into<String>) -> Self {
        self.img_instructions = Some(img.into());
        self
    }

    /// Set instruction image from raw bytes (automatically base64-encoded).
    pub fn with_img_instructions_bytes(mut self, bytes: impl AsRef<[u8]>) -> Self {
        self.img_instructions = Some(STANDARD.encode(bytes.as_ref()));
        self
    }

    /// Get the base64 image body.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Check if case-sensitive mode is enabled.
    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    /// Check if phrase mode is enabled.
    pub fn is_phrase(&self) -> bool {
        self.phrase
    }

    /// Check if math mode is enabled.
    pub fn is_math(&self) -> bool {
        self.math
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_to_text_from_bytes() {
        let bytes = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes
        let task = ImageToText::from_bytes(&bytes);
        // Verify base64 encoding
        assert_eq!(task.body(), STANDARD.encode(&bytes));
        assert!(!task.is_case_sensitive());
        assert!(!task.is_phrase());
        assert!(!task.is_math());
        assert_eq!(task.numeric, 0);
    }

    #[test]
    fn test_image_to_text_from_base64() {
        let task = ImageToText::from_base64("aVZCT1J3MEtHZ29B");
        assert_eq!(task.body(), "aVZCT1J3MEtHZ29B");
    }

    #[test]
    fn test_image_to_text_with_options() {
        let task = ImageToText::from_base64("base64data")
            .case_sensitive()
            .phrase()
            .numbers_only()
            .with_min_length(4)
            .with_max_length(8);

        assert!(task.is_case_sensitive());
        assert!(task.is_phrase());
        assert_eq!(task.numeric, 1);
        assert_eq!(task.min_length, 4);
        assert_eq!(task.max_length, 8);
    }

    #[test]
    fn test_image_to_text_with_website_url() {
        let task = ImageToText::from_base64("base64data").with_website_url("https://example.com");
        assert_eq!(task.website_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_image_to_text_with_module() {
        let task = ImageToText::from_base64("base64data").with_module("number");
        assert_eq!(task.module, Some("number".to_string()));
    }

    #[test]
    fn test_image_to_text_numeric_variants() {
        assert_eq!(ImageToText::from_base64("x").numbers_only().numeric, 1);
        assert_eq!(ImageToText::from_base64("x").letters_only().numeric, 2);
        assert_eq!(
            ImageToText::from_base64("x").numbers_or_letters().numeric,
            3
        );
        assert_eq!(ImageToText::from_base64("x").alphanumeric().numeric, 4);
    }

    #[test]
    fn test_image_to_text_math() {
        let task = ImageToText::from_base64("base64data").math();
        assert!(task.is_math());
    }

    #[test]
    fn test_image_to_text_with_comment() {
        let task = ImageToText::from_base64("base64data").with_comment("Enter red text only");
        assert_eq!(task.comment, Some("Enter red text only".to_string()));
    }

    #[test]
    fn test_image_to_text_with_img_instructions_bytes() {
        let instruction_bytes = vec![0x89, 0x50, 0x4E, 0x47];
        let task =
            ImageToText::from_base64("base64data").with_img_instructions_bytes(&instruction_bytes);
        assert_eq!(
            task.img_instructions,
            Some(STANDARD.encode(&instruction_bytes))
        );
    }

    #[test]
    fn test_image_to_text_clone() {
        let task = ImageToText::from_base64("base64data")
            .case_sensitive()
            .with_module("common");
        let cloned = task.clone();
        assert_eq!(cloned.body, task.body);
        assert_eq!(cloned.case_sensitive, task.case_sensitive);
        assert_eq!(cloned.module, task.module);
    }
}
