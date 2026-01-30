---
name: release
description: Create a new release for the captcha-solvers library. Use when bumping version, updating changelog, and publishing a new tag.
---

# Release Skill

Create and publish a new release of the captcha-solvers library.

## Prerequisites

- Clean working directory (no uncommitted changes)
- On the `master` branch
- All tests passing

## Instructions

### 1. Determine Version Bump

Check recent commits to determine version type:
- **patch** (0.0.x): Bug fixes, documentation updates
- **minor** (0.x.0): New features, non-breaking changes
- **major** (x.0.0): Breaking API changes

### 2. Update Version

Edit `Cargo.toml`:
```toml
[package]
version = "X.Y.Z"  # Update this
```

### 3. Update Changelog

If CHANGELOG.md exists, add entry:
```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features

### Changed
- Changes to existing functionality

### Fixed
- Bug fixes
```

### 4. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to X.Y.Z"
```

### 5. Create and Push Tag

```bash
git tag vX.Y.Z
git push origin master
git push origin vX.Y.Z
```

### 6. Verify Release

The release workflow will automatically:
1. Verify version matches tag
2. Run tests
3. Generate release notes
4. Create GitHub release

Check: https://github.com/rlgrpe/captcha-solvers/releases

## Verification

```bash
# Verify tag exists
git tag -l "vX.Y.Z"

# Check release workflow status
gh run list --workflow=release.yml --limit 1
```
