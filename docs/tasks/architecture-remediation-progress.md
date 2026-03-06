# Architecture Remediation Progress

## Phase 1: Public Contract and Documentation Alignment
- [ ] Fix stale doc blocks referencing `with_provider` and `solve_captcha(task, timeout)`
- [ ] Update `docs/architecture.mermaid` with CapMonster, TurnstileChallenge, WaitRoom, immediate-solution path
- [ ] Add provider capability matrix to README
- [ ] Add note about explicit validation vs silent degradation

## Phase 2: Capability Validation and Task-Mapping Hardening
- [ ] Make `UnsupportedTaskError` field-aware
- [ ] Capsolver: reject ReCaptchaV2 non-enterprise with proxy (silently dropped)
- [ ] Capsolver: reject Turnstile with proxy/pagedata (silently dropped)
- [ ] Capsolver: reject ImageToText OCR fields (silently dropped)
- [ ] RuCaptcha: reject ReCaptchaV3 with proxy (silently dropped)
- [ ] RuCaptcha: reject ReCaptchaV3 enterprise_payload (silently dropped)
- [ ] RuCaptcha: document/decide implicit min_score=0.9 default
- [ ] CapMonster: reject ImageToText OCR fields (silently dropped)
- [ ] CapMonster: align validation with new capability layer
- [ ] Add unit tests for all rejected combinations

## Phase 3: Domain Invariants, Service Lifecycle Tests, and Retry Model
- [ ] Validate `ReCaptchaV3::with_min_score()` range
- [ ] Validate `ImageToText::with_numeric()` range
- [ ] Add service-level tests with mock provider
- [ ] Remove dead provider-side `SolutionTimeout` variants
- [ ] Decide on `should_retry_operation()` future (plan recommends removal)

## Phase 4: Provider Deduplication and Dead-Code Removal
- [ ] Rename `CapsolverProxyFields` to provider-neutral name
- [ ] Extract shared provider transport helper
- [ ] Evaluate consolidating duplicate code

## Phase 5: Final Cleanup and Release Readiness
- [ ] Run cargo fmt, clippy, test
- [ ] Verify examples
- [ ] Update task docs
