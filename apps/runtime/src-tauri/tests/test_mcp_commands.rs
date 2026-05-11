mod helpers;

use runtime_lib::agent::tool_manifest::{ToolCategory, ToolSource};
use runtime_lib::agent::{ToolContext, ToolRegistry};
use runtime_lib::commands::mcp::{
    add_mcp_server_with_registry, remove_mcp_server_with_registry,
    restore_saved_mcp_servers_with_registry,
};
use serde_json::json;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use tempfile::TempDir;

fn python_command() -> String {
    for candidate in ["python", "python3"] {
        if Command::new(candidate)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return candidate.to_string();
        }
    }
    panic!("python or python3 is required for the MCP stdio mock server");
}

fn write_mock_mcp_server() -> (TempDir, String) {
    let tmp = TempDir::new().expect("temp dir");
    let script = tmp.path().join("mock_mcp_server.py");
    std::fs::write(
        &script,
        r#"
import json
import os
import sys

TOOLS = [
    {
        "name": "echo",
        "description": "Echo a message from the mock MCP server",
        "inputSchema": {
            "type": "object",
            "properties": {"message": {"type": "string"}},
            "required": ["message"],
        },
    }
]

def read_message():
    content_length = None
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        line = line.decode("utf-8").strip()
        if not line:
            break
        name, _, value = line.partition(":")
        if name.lower() == "content-length":
            content_length = int(value.strip())
    if content_length is None:
        raise RuntimeError("missing Content-Length")
    return json.loads(sys.stdin.buffer.read(content_length).decode("utf-8"))

def write_message(response):
    body = json.dumps(response).encode("utf-8")
    sys.stdout.buffer.write(f"Content-Length: {len(body)}\r\n\r\n".encode("ascii"))
    sys.stdout.buffer.write(body)
    sys.stdout.buffer.flush()

while True:
    request = read_message()
    if request is None:
        break
    method = request.get("method")
    request_id = request.get("id")
    if method == "notifications/initialized":
        continue
    if method == "initialize":
        response = {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "protocolVersion": "2025-03-26",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "workclaw-test-mcp", "version": "0.0.1"},
            },
        }
    elif method == "tools/list":
        response = {"jsonrpc": "2.0", "id": request_id, "result": {"tools": TOOLS}}
    elif method == "tools/call":
        params = request.get("params", {})
        args = params.get("arguments", {})
        response = {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": f"{os.environ.get('WORKCLAW_MOCK_MCP_LABEL', '')}:{args.get('message', '')}",
                    }
                ]
            },
        }
    else:
        response = {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {"code": -32601, "message": f"unknown method {method}"},
        }
    write_message(response)
"#,
    )
    .expect("write mock server");

    (tmp, script.to_string_lossy().to_string())
}

#[tokio::test(flavor = "multi_thread")]
async fn add_mcp_server_stores_row_and_registers_native_tool() {
    let (pool, _db_tmp) = helpers::setup_test_db().await;
    let registry = Arc::new(ToolRegistry::new());
    let (_server_tmp, script) = write_mock_mcp_server();
    let mut env = HashMap::new();
    env.insert("WORKCLAW_MOCK_MCP_LABEL".to_string(), "native".to_string());

    let id = add_mcp_server_with_registry(
        &pool,
        Arc::clone(&registry),
        "docs".to_string(),
        python_command(),
        vec![script],
        env,
    )
    .await
    .expect("add native MCP server");

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mcp_servers WHERE id = ?")
        .bind(&id)
        .fetch_one(&pool)
        .await
        .expect("count mcp server row");
    assert_eq!(count, 1);

    let tool = registry.get("mcp_docs_echo").expect("registered mcp tool");
    let metadata = tool.metadata();
    assert_eq!(metadata.source, ToolSource::Mcp);
    assert_eq!(metadata.category, ToolCategory::Integration);
    assert_eq!(tool.input_schema()["required"], json!(["message"]));
}

#[tokio::test(flavor = "multi_thread")]
async fn registered_native_mcp_tool_executes_over_stdio() {
    let (pool, _db_tmp) = helpers::setup_test_db().await;
    let registry = Arc::new(ToolRegistry::new());
    let (_server_tmp, script) = write_mock_mcp_server();
    let mut env = HashMap::new();
    env.insert("WORKCLAW_MOCK_MCP_LABEL".to_string(), "native".to_string());

    add_mcp_server_with_registry(
        &pool,
        Arc::clone(&registry),
        "docs".to_string(),
        python_command(),
        vec![script],
        env,
    )
    .await
    .expect("add native MCP server");

    let tool = registry.get("mcp_docs_echo").expect("registered mcp tool");
    let output = tool
        .execute(
            json!({"message": "hello from rust"}),
            &ToolContext::default(),
        )
        .expect("execute native MCP tool");

    assert!(output.contains("native:hello from rust"), "{output}");
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_mcp_server_unregisters_tools_and_deletes_row() {
    let (pool, _db_tmp) = helpers::setup_test_db().await;
    let registry = Arc::new(ToolRegistry::new());
    let (_server_tmp, script) = write_mock_mcp_server();

    let id = add_mcp_server_with_registry(
        &pool,
        Arc::clone(&registry),
        "docs".to_string(),
        python_command(),
        vec![script],
        HashMap::new(),
    )
    .await
    .expect("add native MCP server");
    assert!(registry.get("mcp_docs_echo").is_some());

    remove_mcp_server_with_registry(&pool, Arc::clone(&registry), id)
        .await
        .expect("remove mcp server");

    assert!(registry.get("mcp_docs_echo").is_none());
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mcp_servers")
        .fetch_one(&pool)
        .await
        .expect("count mcp server rows");
    assert_eq!(count, 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn restore_saved_mcp_servers_registers_enabled_servers_without_sidecar_http() {
    let (pool, _db_tmp) = helpers::setup_test_db().await;
    let registry = Arc::new(ToolRegistry::new());
    let (_server_tmp, script) = write_mock_mcp_server();

    sqlx::query(
        "INSERT INTO mcp_servers (id, name, command, args, env, enabled, created_at) VALUES (?, ?, ?, ?, ?, 1, ?)",
    )
    .bind("saved-id")
    .bind("saved")
    .bind(python_command())
    .bind(serde_json::to_string(&vec![script]).expect("args json"))
    .bind("{}")
    .bind("2026-05-11T00:00:00Z")
    .execute(&pool)
    .await
    .expect("insert saved server");

    let restored = restore_saved_mcp_servers_with_registry(&pool, Arc::clone(&registry))
        .await
        .expect("restore saved mcp servers");

    assert_eq!(restored, 1);
    assert!(registry.get("mcp_saved_echo").is_some());
}
