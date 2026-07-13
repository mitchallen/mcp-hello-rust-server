//! The FastMCP-style server: a health check and a demo greeting tool.
//!
//! This is a deliberately small MCP server — a good starting point for a new
//! project or a demo. It exposes two tools:
//!
//!   * `server_info` — health/status of the server.
//!   * `greet` — a friendly greeting in one of a handful of languages,
//!     defaulting to English (e.g. "greet in French" -> "Bonjour!").

use std::sync::OnceLock;
use std::time::Instant;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ErrorData, Json, ServerHandler,
};
use serde::Serialize;

use crate::greetings;

/// Display name reported by `server_info` (override with `APP_NAME`).
pub fn app_name() -> String {
    std::env::var("APP_NAME").unwrap_or_else(|_| "mcp-hello-rust-server".to_string())
}

/// Version reported in the MCP handshake and by `server_info`.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Process start time, captured on first access, used for the uptime readout.
fn start() -> Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    *START.get_or_init(Instant::now)
}

/// Server uptime as `HH:MM:SS`.
fn uptime_hhmmss() -> String {
    let total = start().elapsed().as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        total / 3600,
        (total % 3600) / 60,
        total % 60
    )
}

/// Arguments for the `greet` tool. Both fields are optional.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GreetRequest {
    /// A language name, an alternate spelling, or an ISO code (case-insensitive).
    /// Omit to default to English. Supported: english, spanish, french, german,
    /// italian, portuguese, japanese, hawaiian (e.g. `french`, `Français`, `fr`).
    #[serde(default)]
    pub language: Option<String>,
    /// Optional name to personalize the message (e.g. "Bonjour, Alice!").
    #[serde(default)]
    pub name: Option<String>,
}

/// The `greet` tool's structured result: `{ language, greeting, message }`.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct GreetResult {
    pub language: String,
    pub greeting: String,
    pub message: String,
}

/// The `server_info` tool's structured result.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ServerInfoResult {
    pub status: String,
    pub app: String,
    pub version: String,
    pub uptime: String,
    pub languages: Vec<String>,
    pub default_language: String,
    pub source: String,
    pub author: String,
}

/// The MCP server. Holds the generated tool router.
#[derive(Clone)]
pub struct Hello {
    // Read by the code the #[tool_handler] macro generates (via Clone), which
    // the dead-code lint can't see through — hence the allow.
    #[allow(dead_code)]
    tool_router: ToolRouter<Hello>,
}

impl Default for Hello {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl Hello {
    pub fn new() -> Self {
        // Capture the start time as soon as a server is constructed.
        let _ = start();
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Health/status of the server.
    #[tool(
        description = "Health/status of the server: app name, version, uptime, and supported greeting languages."
    )]
    async fn server_info(&self) -> Json<ServerInfoResult> {
        Json(ServerInfoResult {
            status: "OK".to_string(),
            app: app_name(),
            version: APP_VERSION.to_string(),
            uptime: uptime_hhmmss(),
            languages: greetings::languages()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            default_language: greetings::DEFAULT_LANGUAGE.to_string(),
            source: "https://github.com/mitchallen/mcp-hello-rust-server".to_string(),
            author: "Mitch Allen (https://mitchallen.com)".to_string(),
        })
    }

    /// Return a friendly greeting in the requested language (default English).
    #[tool(
        description = "Return a friendly greeting in the requested language (default English). \
        `language` accepts a language name, alternate spelling, or ISO code (english, spanish, \
        french, german, italian, portuguese, japanese, hawaiian). Optional `name` personalizes \
        the message. Returns { language, greeting, message }."
    )]
    async fn greet(
        &self,
        Parameters(GreetRequest { language, name }): Parameters<GreetRequest>,
    ) -> Result<Json<GreetResult>, ErrorData> {
        match greetings::greet(language.as_deref(), name.as_deref()) {
            Ok(g) => Ok(Json(GreetResult {
                language: g.language,
                greeting: g.greeting,
                message: g.message,
            })),
            // Unknown language -> surface as a client error (invalid params).
            Err(msg) => Err(ErrorData::invalid_params(msg, None)),
        }
    }
}

#[tool_handler]
impl ServerHandler for Hello {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(app_name(), APP_VERSION))
            .with_instructions(
                "A minimal demo MCP server. Use server_info for a health/status check, and \
                greet to get a friendly greeting in a given language (english, spanish, french, \
                german, italian, portuguese, japanese, or hawaiian; defaults to english). For \
                example, 'greet in French' returns 'Bonjour!'.",
            )
    }
}
