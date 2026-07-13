# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-13

### Added

- Initial release: a minimal MCP server built with **Rust** and the
  [`rmcp`](https://crates.io/crates/rmcp) SDK, exposing two tools —
  - `server_info()` — a health/status check reporting the app name, version,
    uptime, supported greeting languages, and default language.
  - `greet(language?, name?)` — a friendly greeting in one of a handful of
    languages (english, spanish, french, german, italian, portuguese, japanese,
    hawaiian), defaulting to English. Accepts a language name, alternate
    spelling, or ISO code (case-insensitive), and an optional `name` to
    personalize the message.
- Serves over **stdio** (default) or **streamable HTTP** (`MCP_TRANSPORT=http`),
  selected at runtime; the MCP endpoint is `/mcp`.
- A test suite driven through an in-memory `rmcp` client over a
  `tokio::io::duplex` pipe (`tests/server.rs`), plus unit tests for the greeting
  resolver/builder (`src/greetings.rs`).
- Multi-stage **Docker** build producing a fully-static musl binary on a
  distroless Chainguard/Wolfi `static` base — a ~10 MB image that runs as a
  non-root user and scans **0 known vulnerabilities** with Trivy.
- **CI**: a `ci` workflow (fmt + clippy + tests), an `image-scan` workflow that
  fails the build on fixable CRITICAL/HIGH image vulnerabilities, a `cargo-audit`
  workflow scanning Rust dependencies against the RustSec advisory DB, a daily
  `scan-scheduled` re-scan of the published `:latest` image, and GHCR + Docker
  Hub `publish` workflows gated behind a pre-push Trivy scan.
- A Dependabot config opening weekly update PRs for Cargo dependencies, the
  Docker base image, and GitHub Actions, with low-risk updates auto-merged once
  CI passes.

[unreleased]: https://github.com/mitchallen/mcp-hello-rust-server/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mitchallen/mcp-hello-rust-server/releases/tag/v0.1.0
