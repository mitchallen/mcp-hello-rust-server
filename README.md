# mcp-hello-rust-server

[![ci](https://github.com/mitchallen/mcp-hello-rust-server/actions/workflows/ci.yml/badge.svg)](https://github.com/mitchallen/mcp-hello-rust-server/actions/workflows/ci.yml) [![image-scan](https://github.com/mitchallen/mcp-hello-rust-server/actions/workflows/image-scan.yml/badge.svg)](https://github.com/mitchallen/mcp-hello-rust-server/actions/workflows/image-scan.yml) [![cargo-audit](https://github.com/mitchallen/mcp-hello-rust-server/actions/workflows/cargo-audit.yml/badge.svg)](https://github.com/mitchallen/mcp-hello-rust-server/actions/workflows/cargo-audit.yml)

A minimal [MCP](https://modelcontextprotocol.io) server built with **Rust** and
the [`rmcp`](https://crates.io/crates/rmcp) SDK — a good starting point for a new
server or a demo. It exposes just two tools:

- **`server_info`** — a health/status check.
- **`greet`** — a friendly greeting in one of a handful of languages, defaulting
  to English. Ask it to "greet in French" and it replies `Bonjour!`.

Built with **Rust**, **[rmcp](https://crates.io/crates/rmcp)**, **cargo**, and
**make**. It is the Rust port of the sibling Python
[`mcp-hello-server`](../mcp-hello-server), following the official MCP
[Build a server (Rust)](https://modelcontextprotocol.io/docs/develop/build-server#rust)
reference. The Docker image is a fully-static binary on a distroless base — about
**10 MB** and **0 known vulnerabilities**.

* * *

## Quick start — demo an MCP server in 2 minutes

New to MCP? This is a tiny, safe server for **seeing how an MCP client discovers
and calls tools**. Every tool is a harmless in-memory lookup, so it's a good
sandbox. All you need is **[Docker](https://docs.docker.com/get-docker/)** and an
MCP client — the steps below use **[Claude Code](https://claude.com/claude-code)**
and the published Docker image (nothing to build or install).

> **Already running the Python [`mcp-hello-server`](../mcp-hello-server)?** It
> registers under the alias `hello` and exposes the same `server_info` / `greet`
> tools, so it's easy to mix up with this one — you might test the Python server
> while thinking you're testing the Rust one. Remove it first so your client only
> talks to `hello-rust`:
>
> ```sh
> claude mcp list                # see what's registered (look for "hello")
> claude mcp remove hello        # remove the Python server (its default alias)
> ```
>
> Use `--scope user` / `--scope project` if it was added at that scope, e.g.
> `claude mcp remove hello --scope user`. In Claude Desktop, delete the `hello`
> entry from `mcpServers` in `claude_desktop_config.json` instead and restart.

**1. Add the server.** Claude Code launches the container per session and talks
to it over stdio:

```sh
claude mcp add hello-rust -- docker run -i --rm -e MCP_TRANSPORT=stdio ghcr.io/mitchallen/mcp-hello-rust-server:latest
```

**2. Confirm it connected:**

```sh
claude mcp list        # "hello-rust" should report ✔ Connected
```

**3. Ask in plain language** — Claude discovers the tools and picks one (the tool
it calls is in parentheses):

- "Is the hello server up? What version is it?" → (`server_info`)
- "Greet me in French." → (`greet` → **Bonjour!**)
- "Say hello in Japanese to Alice." → (`greet` → **こんにちは (Konnichiwa), Alice!**)
- "What languages can you greet in?" → (`server_info`, reads `languages`)

That round trip — the client listing tools, then calling one with arguments and
getting structured JSON back — *is* MCP.

**4. Remove it when you're done:**

```sh
claude mcp remove hello-rust
```

> **Prefer HTTP?** Run it as a long-lived server instead:
> ```sh
> docker run --rm -p 8000:8000 ghcr.io/mitchallen/mcp-hello-rust-server:latest
> claude mcp add --transport http hello-rust http://localhost:8000/mcp
> ```

* * *

## Tools

| Tool                        | Purpose                                                        |
| --------------------------- | ------------------------------------------------------------- |
| `server_info()`             | Health/status: app name, version, uptime, supported languages |
| `greet(language?, name?)`   | Greeting in `language` (default English); optional `name`     |

### `greet`

`greet` takes two optional arguments:

- **`language`** — a language name, an alternate spelling, or an ISO code
  (case-insensitive). Omit it to default to English. Supported: `english`,
  `spanish`, `french`, `german`, `italian`, `portuguese`, `japanese`,
  `hawaiian` (e.g. `french`, `Français`, or `fr` all work).
- **`name`** — optional; personalizes the message (`Bonjour, Alice!`).

It returns `{ language, greeting, message }`:

```jsonc
// greet(language="french")
{ "language": "french", "greeting": "Bonjour", "message": "Bonjour!" }

// greet(language="spanish", name="Alice")
{ "language": "spanish", "greeting": "Hola", "message": "Hola, Alice!" }

// greet()  -> { "language": "english", "greeting": "Hello", "message": "Hello!" }
```

An unknown language returns an error listing the supported set.

### Add a language

Add a row to `GREETINGS` in `src/greetings.rs` (and, optionally, an alias / ISO
code in `ALIASES`). `server_info` reports the supported set automatically.

* * *

## Quick start (from source)

Requires a [Rust toolchain](https://www.rust-lang.org/tools/install) (1.85+).

```sh
make build       # cargo build --release
make test        # run the test suite
make run         # run the server over stdio
```

`make help` lists every target.

* * *

## Running the server

### stdio (default — for MCP clients that launch the server)

```sh
cargo run --release
# or
make run
```

### Streamable HTTP (for networked clients / containers)

```sh
make run-http            # PORT defaults to 8000
PORT=9000 make run-http
```

The MCP endpoint is served at `/mcp`.

* * *

## Configuration

All configuration is via environment variables:

| Variable        | Default                 | Purpose                                    |
| --------------- | ----------------------- | ------------------------------------------ |
| `APP_NAME`      | `mcp-hello-rust-server` | Name reported by `server_info`             |
| `MCP_TRANSPORT` | `stdio`                 | `stdio` or `http`                          |
| `HOST`          | `127.0.0.1`             | Bind address for `http`                    |
| `PORT`          | `8000`                  | Bind port for `http`                       |

* * *

## Using with an MCP client — local development (from source)

Point a stdio-based client (e.g. Claude Desktop, Claude Code) at the release
binary. With Claude Code, from the project directory:

```sh
make build
claude mcp add hello-rust -- "$PWD/target/release/mcp-hello-rust-server"
```

Confirm it's connected with `claude mcp list` (or `/mcp` inside a session).

### Example prompts (Claude Code)

Once the server is added, just ask in plain language — Claude picks the right
tool. The tool it invokes is shown in parentheses.

- "Is the hello server up? What version is it?" → (`server_info`)
- "Greet me." → (`greet`, defaults to English → "Hello!")
- "Greet in French." → (`greet` with `language="french"` → "Bonjour!")
- "Say hello in Japanese to Alice." → (`greet` with `language="japanese"`, `name="Alice"`)
- "What languages can you greet in?" → (`server_info`, then read `languages`)

* * *

## Using a published image

The image is published to two registries:

- **GitHub Container Registry:** `ghcr.io/mitchallen/mcp-hello-rust-server`
- **Docker Hub:** `mitchallen/mcp-hello-rust-server`

### Option A — Docker image, client launches it (stdio)

The client starts a fresh container per session and talks to it over stdio. Use
`-i` (keep stdin open) and force the stdio transport, since the image defaults to
HTTP:

```jsonc
{
  "mcpServers": {
    "hello-rust": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "-e", "MCP_TRANSPORT=stdio",
               "ghcr.io/mitchallen/mcp-hello-rust-server:latest"]
    }
  }
}
```

Claude Code equivalent:

```sh
claude mcp add hello-rust -- docker run -i --rm -e MCP_TRANSPORT=stdio ghcr.io/mitchallen/mcp-hello-rust-server:latest
```

(Pin a version like `:0.1.0` in place of `:latest` for a reproducible setup.)

### Option B — Long-running container over HTTP

The image serves HTTP by default. Start it once, then point an HTTP-capable
client at it:

```sh
docker run -d --rm -p 8000:8000 --name mcp-hello-rust ghcr.io/mitchallen/mcp-hello-rust-server:latest
claude mcp add --transport http hello-rust http://localhost:8000/mcp
```

For clients that only speak **stdio**, bridge to the HTTP endpoint with
[`mcp-remote`](https://www.npmjs.com/package/mcp-remote):

```jsonc
{
  "mcpServers": {
    "hello-rust": {
      "command": "npx",
      "args": ["-y", "mcp-remote", "http://localhost:8000/mcp"]
    }
  }
}
```

Notes for remote use:

- Prefer **HTTPS** so traffic is encrypted in transit.
- This server ships **no authentication**. If you expose it beyond localhost, put
  it behind a reverse proxy, gateway, or network policy.
- The endpoint path is `/mcp`.

* * *

## Docker

Published multi-platform (`linux/amd64`, `linux/arm64`) images run the server
over **streamable HTTP** by default (`MCP_TRANSPORT=http`, `HOST=0.0.0.0`,
`PORT=8000`) so they're reachable on a published port.

The build compiles a **fully-static musl binary** on `rust:1-alpine` and copies
it onto a distroless **[Chainguard/Wolfi](https://images.chainguard.dev) `static`
base** — no shell, no package manager, runs as a non-root user, ~10 MB, and scans
**0 known vulnerabilities**. Every build is gated by a Trivy scan (fails on
fixable CRITICAL/HIGH); the Rust dependency tree is separately scanned with
`cargo-audit`, and the published `:latest` is re-scanned daily — see
[Security scanning](#security-scanning).

### Pull and run

```sh
docker pull ghcr.io/mitchallen/mcp-hello-rust-server:latest
docker run --rm -p 8000:8000 --name mcp-hello-rust ghcr.io/mitchallen/mcp-hello-rust-server:latest
```

Then connect an HTTP MCP client to `http://localhost:8000/mcp`.

### Test a published release with make

Convenience targets pull and run the **published** image locally — handy for
smoke-testing a release without a local build:

```sh
make docker-test               # up + smoke + down in one shot (exits non-zero on failure)

make docker-up                 # pull + run ghcr.io/mitchallen latest, detached
make docker-smoke              # MCP `initialize` handshake — passes if the server responds
make docker-down               # stop it

make docker-up TAG=0.1.0                         # pin a version
make docker-up REGISTRY=docker.io/mitchallen     # pull from Docker Hub instead
make docker-up HTTP_PORT=9000                    # publish on a different host port
```

### Build locally

```sh
make docker-build        # docker build -t mcp-hello-rust-server .
make docker-run          # serves http on localhost:8000
make scan                # Trivy scan of the local image (fixable CRITICAL/HIGH fail)
```

* * *

## Security scanning

Two complementary gates catch vulnerabilities, both reproducible locally:

- **`image-scan`** (`make scan`) — Trivy scans the built container image and
  fails the build on **fixable** CRITICAL/HIGH vulnerabilities. This is the OS /
  base-image layer.
- **`cargo-audit`** — scans `Cargo.lock` against the
  [RustSec advisory DB](https://rustsec.org/) for vulnerable crates compiled into
  the binary (which an image scan can't see inside a static binary). Runs on
  every push/PR and daily.
- **`scan-scheduled`** re-scans the published `:latest` image daily and uploads
  results to the GitHub Security tab, catching CVEs disclosed after build time.
- **Dependabot** opens weekly PRs for Cargo deps, the Docker base image, and
  GitHub Actions; low-risk updates auto-merge once CI passes.

* * *

## CI / Publish

Workflows live in `.github/workflows/`:

- **`ci`** — on every push/PR to `main`: `cargo fmt --check`, `cargo clippy -D
  warnings`, and `cargo test`.
- **`image-scan`** / **`cargo-audit`** / **`scan-scheduled`** — vulnerability
  scanning (see above).
- **`publish`** / **`publish-dockerhub`** — triggered by pushing a `v*` tag.
  Build a multi-platform image, Trivy-scan it, push it to GHCR and Docker Hub,
  then run `make docker-test` against the just-published image. The Docker Hub
  job needs `DOCKERHUB_USERNAME` / `DOCKERHUB_TOKEN` repository secrets.

To cut a release, use the `release` target — it bumps `version` in `Cargo.toml`
(and `Cargo.lock`), commits, tags, pushes, and creates the GitHub Release from
the `CHANGELOG.md` section, which triggers both publish workflows:

```sh
make release              # patch bump (default)
make release BUMP=minor   # or minor / major
```

The target refuses to run unless the working tree is clean, you're on `main`, and
`CHANGELOG.md` already has a `## [X.Y.Z]` section for the new version.

* * *

## Development

- Source: `src/`
  - `greetings.rs` — greeting data + language resolution (`greet`), with unit tests
  - `server.rs` — the `rmcp` tools (`#[tool_router]` / `#[tool]`) + `ServerHandler`
  - `main.rs` — the binary; transport wiring (stdio / HTTP)
  - `lib.rs` — exposes the modules to the test crate
- Tests: `tests/server.rs` drives the tools through an **in-memory `rmcp` client**
  over a `tokio::io::duplex` pipe (no network, no subprocess); `src/greetings.rs`
  has `#[cfg(test)]` unit tests for the resolver/builder. Run everything with
  `make test`, or the full CI gate with `make check` (fmt + clippy + test).
- **Dependencies:** `Cargo.lock` is committed and the Docker build compiles from
  it. Run `make lock` (`cargo generate-lockfile`) after changing dependencies.

* * *

## License

MIT © Mitch Allen
