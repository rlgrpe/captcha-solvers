# Architecture Remediation Progress

## Phase 1: Public Contract and Documentation Alignment
- [x] Fix stale doc blocks referencing `with_provider` and `solve_captcha(task, timeout)`
- [x] Update `docs/architecture.mermaid` with CapMonster, TurnstileChallenge, WaitRoom, immediate-solution path
- [x] Add provider capability matrix to README
- [x] Add note about explicit validation vs silent degradation

## Phase 2: Capability Validation and Task-Mapping Hardening
- [x] Make `UnsupportedTaskError` field-aware
- [x] Capsolver: reject ReCaptchaV2 non-enterprise with proxy (silently dropped)
- [x] Capsolver: reject Turnstile with proxy/pagedata (silently dropped)
- [x] Capsolver: reject ImageToText OCR fields (silently dropped)
- [x] RuCaptcha: reject ReCaptchaV3 with proxy (silently dropped)
- [x] RuCaptcha: reject ReCaptchaV3 enterprise_payload (silently dropped)
- [x] RuCaptcha: document/decide implicit min_score=0.9 default (kept, documented)
- [x] CapMonster: reject ReCaptchaV3 with proxy/api_domain/enterprise_payload
- [x] CapMonster: reject ImageToText OCR fields (silently dropped)
- [x] Add unit tests for all rejected combinations (9 new tests)

## Phase 3: Domain Invariants, Service Lifecycle Tests, and Retry Model
- [x] Validate `ReCaptchaV3::with_min_score()` range (0.1..=0.9)
- [x] Validate `ImageToText::with_numeric()` range (0..=4)
- [x] Add service-level tests with mock provider (6 tests)
- [x] Remove dead provider-side `SolutionTimeout` variants (all 3 providers)
- [x] Decide on `should_retry_operation()` — kept (used in ServiceError + tracing)

## Phase 4: Provider Deduplication and Dead-Code Removal
- [x] Rename `CapsolverProxyFields` → `ApiProxyFields` (provider-neutral)
- [x] Rename `into_capsolver_fields()` → `into_api_proxy_fields()`
- [x] Rename `as_capsolver_str()` → `as_api_str()`
- [x] Evaluate shared transport extraction — deferred (macro-heavy for marginal benefit)

## Phase 5: Final Cleanup and Release Readiness
- [x] Run cargo fmt --check (clean)
- [x] Run cargo clippy --all-features -- -D warnings (clean)
- [x] Run cargo test --all-features (180 tests pass)
- [x] Run cargo test --doc (23 pass)
- [x] Verify examples compile
- [x] Update task docs

## Summary of Changes

| Metric | Before | After |
|--------|--------|-------|
| Unit tests | 160 | 180 |
| Silent field drops | ~15 | 0 |
| Dead code variants | 3 (`SolutionTimeout`) | 0 |
| Provider-leaking names | `CapsolverProxyFields` | `ApiProxyFields` |

### Commits
1. `docs: align public API docs and architecture diagram`
2. `feat: add capability validation to prevent silent field dropping`
3. `test: add service lifecycle tests and task validation`
4. `refactor: rename provider-leaking proxy types to neutral names`
