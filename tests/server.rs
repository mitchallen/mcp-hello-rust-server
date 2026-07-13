//! Tests for the MCP tool layer, driven through an in-memory client.
//!
//! A server ([`Hello`]) and a trivial client (`()`) are connected over an
//! in-process `tokio::io::duplex` pipe — the Rust analog of the Python suite's
//! in-memory FastMCP client. No network, no subprocess.

use rmcp::{model::CallToolRequestParams, service::RunningService, RoleClient, ServiceExt};
use serde_json::{json, Map, Value};

use mcp_hello_rust_server::server::Hello;

/// Spin up the server on one end of a duplex pipe and return a connected client.
async fn connect() -> RunningService<RoleClient, ()> {
    let (client_io, server_io) = tokio::io::duplex(4096);
    tokio::spawn(async move {
        let server = Hello::new()
            .serve(server_io)
            .await
            .expect("server handshake");
        let _ = server.waiting().await;
    });
    ().serve(client_io).await.expect("client handshake")
}

/// Call a tool and return its structured JSON content as an object.
async fn call(
    client: &RunningService<RoleClient, ()>,
    name: &'static str,
    args: Value,
) -> Map<String, Value> {
    let mut param = CallToolRequestParams::new(name);
    match args {
        Value::Object(m) => param = param.with_arguments(m),
        Value::Null => {}
        other => panic!("args must be a JSON object, got {other}"),
    }
    let result = client.call_tool(param).await.expect("call_tool");
    result
        .structured_content
        .and_then(|v| v.as_object().cloned())
        .expect("tool returned structured content")
}

#[tokio::test]
async fn tools_are_registered() {
    let client = connect().await;
    let names: std::collections::HashSet<String> = client
        .list_all_tools()
        .await
        .expect("list_tools")
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    assert_eq!(
        names,
        ["server_info", "greet"]
            .into_iter()
            .map(String::from)
            .collect()
    );
    client.cancel().await.ok();
}

#[tokio::test]
async fn server_info_reports_status_and_metadata() {
    let client = connect().await;
    let info = call(&client, "server_info", Value::Null).await;
    assert_eq!(info["status"], "OK");
    assert!(info["languages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l == "english"));
    assert_eq!(info["default_language"], "english");
    assert_eq!(
        info["source"],
        "https://github.com/mitchallen/mcp-hello-rust-server"
    );
    assert_eq!(info["author"], "Mitch Allen (https://mitchallen.com)");
    assert!(!info["version"].as_str().unwrap().is_empty());
    client.cancel().await.ok();
}

#[tokio::test]
async fn greet_defaults_to_english() {
    let client = connect().await;
    let g = call(&client, "greet", json!({})).await;
    assert_eq!(g["language"], "english");
    assert_eq!(g["message"], "Hello!");
    client.cancel().await.ok();
}

#[tokio::test]
async fn greet_in_french() {
    let client = connect().await;
    let g = call(&client, "greet", json!({ "language": "French" })).await;
    assert_eq!(g["language"], "french");
    assert_eq!(g["message"], "Bonjour!");
    client.cancel().await.ok();
}

#[tokio::test]
async fn greet_personalized() {
    let client = connect().await;
    let g = call(
        &client,
        "greet",
        json!({ "language": "spanish", "name": "Alice" }),
    )
    .await;
    assert_eq!(g["language"], "spanish");
    assert_eq!(g["message"], "Hola, Alice!");
    client.cancel().await.ok();
}

#[tokio::test]
async fn greet_unknown_language_errors() {
    let client = connect().await;
    let mut args = Map::new();
    args.insert("language".into(), json!("klingon"));
    let err = client
        .call_tool(CallToolRequestParams::new("greet").with_arguments(args))
        .await
        .expect_err("unknown language should error");
    assert!(err.to_string().contains("unknown language"), "got: {err}");
    client.cancel().await.ok();
}
