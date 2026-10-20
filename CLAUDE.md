# mcp-hello-rust-server — notes for Claude

A minimal MCP server built with **Rust** and the
[`rmcp`](https://crates.io/crates/rmcp) SDK — a good starting point for a new
server or a demo. It exposes two tools: `server_info` (a health/status check)
and `greet` (a friendly greeting in one of a handful of languages, defaulting to
English). Built with **cargo** and **make**; a multi-stage Docker build produces
a fully-static musl binary on a distroless Chainguard/Wolfi `static` base
(`cgr.dev/chainguard/static`) for a ~10 MB, near-zero-CVE image. It is the Rust
port of the sibling Python [`mcp-hello-server`](../mcp-hello-server), following
the official MCP "Build a server (Rust)" reference.

## Layout

- `src/greetings.rs` — greeting data (`GREETINGS`), language resolution (names,
  aliases, ISO codes), and the `greet()` builder, with `#[cfg(test)]` unit tests.
- `src/server.rs` — the `rmcp` server (`Hello`), its tools (`server_info`,
  `greet`) defined via `#[tool_router]` / `#[tool]`, and the `ServerHandler`
  impl (`#[tool_handler]`). Tools return `Json<T>` for structured content.
- `src/main.rs` — the binary entry point; picks the transport from
  `MCP_TRANSPORT` and wires up stdio or streamable HTTP (axum).
- `src/lib.rs` — exposes `greetings` + `server` so `tests/` can drive the server.
- `tests/server.rs` — integration tests connecting a client (`()`) and the
  server over a `tokio::io::duplex` pipe (the analog of the Python suite's
  in-memory FastMCP client). No network, no subprocess.

## Conventions

- **Build / deps:** cargo. `Cargo.lock` is committed and the Dockerfile builds
  `--release` from it. `make build` = `cargo build --release`.
- **Running:** `make run` (stdio, the default MCP transport), `make run-http`
  (streamable HTTP on `PORT`, default 8000). The transport is chosen by
  `MCP_TRANSPORT` (`stdio` | `http`); the HTTP endpoint is `/mcp`.
- **Tests / lint:** `make test` (`cargo test`), `make check` (fmt --check +
  clippy -D warnings + test) — the same gate CI runs.
- **Adding a language:** add a row to `GREETINGS` in `greetings.rs` (and,
  optionally, an alias / ISO code in `ALIASES`). `server_info` reports the
  supported set automatically.
- **Docker:** the image defaults to HTTP transport (`MCP_TRANSPORT=http`,
  `HOST=0.0.0.0`, `PORT=8000`) so it's reachable on a published port. The binary
  is built static on `rust:1-alpine` (musl) and copied onto
  `cgr.dev/chainguard/static`, which has no shell / package manager and runs as
  the non-root `nonroot` user (uid 65532). `make scan` should report 0
  CRITICAL/HIGH.
- **Releasing:** `make release` (`BUMP=patch|minor|major`, default patch) bumps
  `version` in `Cargo.toml` (+ `Cargo.lock`), commits, tags `vX.Y.Z`, pushes, and
  creates the matching GitHub Release from the `CHANGELOG.md` section. The tag
  triggers the GHCR + Docker Hub publish workflows. It refuses to run unless the
  tree is clean, you're on `main`, and `CHANGELOG.md` already has the new
  version's section — add that (Keep a Changelog format, top of the file) first.
- **Branch protection:** `main` requires the `test`, `scan` and `audit` checks
  (the `ci`, `image-scan` and `cargo-audit` workflows), and the repo has
  auto-merge enabled. Both were set up on 2026-10-12. Before that `main` was
  unprotected and auto-merge was disabled, so `dependabot-auto-merge.yml`'s
  `gh pr merge --auto` merged Dependabot PRs immediately instead of waiting for
  CI (#25 and #26 landed that way). `gh pr merge --auto` only waits for
  *required* checks, so keep this list in step with the CI jobs.
  `enforce_admins` stays **off** on purpose: `make release` pushes the version
  commit straight to `main`, which admin enforcement would block (expect a
  "Bypassed rule violations" notice on that push).

## Tools

| Tool                       | Purpose                                                  |
| -------------------------- | -------------------------------------------------------- |
| `server_info()`            | Health/status: app name, version, uptime, languages.     |
| `greet(language?, name?)`  | Greeting in a language (default English); optional name. |

Supported languages: `english`, `spanish`, `french`, `german`, `italian`,
`portuguese`, `japanese`, `hawaiian`. Lookups accept aliases / ISO codes
(`fr`, `Français`, …) case-insensitively.

## Security scanning

Two complementary gates, both wired into CI and reproducible locally:

- **`image-scan`** (`make scan`) — in CI, first starts the built image and runs
  `make docker-smoke` (an MCP `initialize`) against it, so an image that builds
  but crashes at startup fails. Then Trivy scans the built image and fails on
  fixable CRITICAL/HIGH. Covers the OS layer of the runtime image.
- **`cargo-audit`** — `rustsec/audit-check` scans `Cargo.lock` against the
  RustSec advisory DB. Covers the crates compiled into the binary (which the
  image scan can't see inside a static binary). Also runs daily.
- **`scan-scheduled`** re-scans the published `:latest` daily so CVEs disclosed
  after build time still surface in the Security tab.

## Gotchas

- **`rmcp` model structs are `#[non_exhaustive]`** (e.g. `ServerInfo`,
  `Implementation`, `CallToolRequestParams`), so they cannot be built with struct
  literals from this crate — use their `::new(...)` / `.with_*(...)` builders.
- **`Parameters` lives at `rmcp::handler::server::wrapper::Parameters`** (not
  `...::tool::Parameters`); `Json` is re-exported at `rmcp::Json`.
- **Docker dep-cache stub:** the Dockerfile builds a stub crate first to cache
  dependencies, then `touch`es the real `src/*.rs` before rebuilding. Without the
  `touch`, cargo sees the COPYed sources as older than the cached stub artifacts
  and ships the empty stub binary (the container then exits immediately).
- **The `tool_router` field carries `#[allow(dead_code)]`** — it's only read by
  the code the `#[tool_handler]` macro generates, which the dead-code lint can't
  see through.
- **Logs go to stderr** (`tracing_subscriber` with `with_writer(stderr)`): the
  stdio transport owns stdout for the JSON-RPC stream, so a stray stdout write
  would corrupt the protocol.

## Releases — lessons from 2026-10-20

- **Is a release due?** Compare `main` against the latest tag and ignore
  `.github/` and `*.md` — CI and docs changes don't reach the image.
  `Cargo.lock` changes count: crates are compiled into the static binary, so
  even a lockfile-only patch (`serde_json` 1.0.151 → 1.0.154, #35) needs a
  release for the image to carry it — that was v0.2.3. Dependabot auto-merges
  land on `main` without a release, so a production bump sits unshipped until
  someone cuts one.
- **`make release` does not touch the changelog links.** Add the `## [X.Y.Z] -
  YYYY-MM-DD` section *and* the footer links — point `[unreleased]` at
  `compare/vX.Y.Z...HEAD` and add `[X.Y.Z]: …/compare/vPREV...vX.Y.Z` — in one
  `docs: changelog for vX.Y.Z` commit to `main`, then run `make release`. It
  refuses to run without the section, creates the GitHub Release from it, and
  the `main` push shows a "Bypassed rule violations" notice (admins are not
  enforced).
- **Publish time:** 19–27 min (v0.2.1–v0.2.3), mostly the `linux/arm64` build
  under QEMU. That's normal. Compare against `gh run list --workflow
  publish.yml` before treating a slow run as hung.
- **Verify what shipped, not just the green runs.** `docker buildx imagetools
  inspect <image>:X.Y.Z` on both registries should list `linux/amd64` and
  `linux/arm64`. Then `make docker-test TAG=X.Y.Z` pulls the *published* image
  and runs the MCP `initialize` smoke check, and the reply's `serverInfo` should
  carry the released version — v0.2.3 replied
  `{"name":"mcp-hello-rust-server","version":"0.2.3"}`.
- **The required `scan` check now starts the container** (see above), so a PR
  whose image builds but won't serve MCP is caught before release, not after.
- **Sibling conventions differ** — release flows, merge styles and Dependabot
  schedules vary across the eight active repos in `../`. random-server's
  CLAUDE.md ("Sibling projects") keeps the comparison; check it, or the
  sibling's own CLAUDE.md, before porting a fix.
