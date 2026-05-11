use crate::agent::tool_manifest::{ToolCategory, ToolMetadata, ToolSource};
use crate::agent::types::{Tool, ToolContext};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::{timeout, Duration};

const MCP_PROTOCOL_VERSION: &str = "2025-03-26";
const MCP_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone)]
pub struct NativeMcpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeMcpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct NativeMcpTool {
    tool_name: String,
    tool_description: String,
    input_schema: Value,
    server: NativeMcpServerConfig,
    mcp_tool_name: String,
}

impl NativeMcpTool {
    pub fn new(
        tool_name: String,
        tool_description: String,
        input_schema: Value,
        server: NativeMcpServerConfig,
        mcp_tool_name: String,
    ) -> Self {
        Self {
            tool_name,
            tool_description,
            input_schema,
            server,
            mcp_tool_name,
        }
    }
}

impl Tool for NativeMcpTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_description
    }

    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        let server = self.server.clone();
        let tool_name = self.mcp_tool_name.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| anyhow!("native MCP call failed: failed to start runtime: {e}"))?;
            runtime.block_on(call_native_mcp_tool(&server, &tool_name, input))
        })
        .join()
        .map_err(|_| anyhow!("native MCP call failed: worker thread panicked"))?
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            display_name: Some(self.tool_name.clone()),
            category: ToolCategory::Integration,
            read_only: false,
            destructive: false,
            concurrency_safe: false,
            open_world: true,
            requires_approval: false,
            source: ToolSource::Mcp,
        }
    }
}

pub async fn list_native_mcp_tools(
    server: &NativeMcpServerConfig,
) -> Result<Vec<NativeMcpToolDefinition>> {
    let mut session = NativeMcpSession::connect(server)
        .await
        .map_err(|e| anyhow!("native MCP connection failed for {}: {e}", server.name))?;
    let response = session
        .request("tools/list", json!({}))
        .await
        .map_err(|e| anyhow!("native MCP list failed for {}: {e}", server.name))?;
    session.close().await;

    let tools = response
        .get("tools")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            anyhow!(
                "native MCP list failed for {}: missing tools array",
                server.name
            )
        })?;

    Ok(tools
        .iter()
        .filter_map(|tool| {
            let name = tool.get("name")?.as_str()?.to_string();
            let description = tool
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let input_schema = tool
                .get("inputSchema")
                .cloned()
                .unwrap_or_else(default_input_schema);
            Some(NativeMcpToolDefinition {
                name,
                description,
                input_schema,
            })
        })
        .collect())
}

pub async fn call_native_mcp_tool(
    server: &NativeMcpServerConfig,
    tool_name: &str,
    arguments: Value,
) -> Result<String> {
    let mut session = NativeMcpSession::connect(server)
        .await
        .map_err(|e| anyhow!("native MCP connection failed for {}: {e}", server.name))?;
    let response = session
        .request(
            "tools/call",
            json!({
                "name": tool_name,
                "arguments": arguments,
            }),
        )
        .await
        .map_err(|e| {
            anyhow!(
                "native MCP call failed for {}.{}: {e}",
                server.name,
                tool_name
            )
        })?;
    session.close().await;
    Ok(mcp_call_result_to_string(response))
}

fn default_input_schema() -> Value {
    json!({"type": "object", "properties": {}})
}

fn mcp_call_result_to_string(value: Value) -> String {
    if let Some(content) = value.get("content") {
        if let Some(text) = content.as_str() {
            return text.to_string();
        }
        if let Some(items) = content.as_array() {
            let text = items
                .iter()
                .filter_map(|item| item.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>();
            if !text.is_empty() {
                return text.join("\n");
            }
        }
    }

    if let Some(output) = value.get("output").and_then(Value::as_str) {
        return output.to_string();
    }

    serde_json::to_string(&value).unwrap_or_default()
}

struct NativeMcpSession {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl NativeMcpSession {
    async fn connect(server: &NativeMcpServerConfig) -> Result<Self> {
        let mut command = Command::new(&server.command);
        command
            .args(&server.args)
            .envs(&server.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let mut child = command.spawn().map_err(|e| {
            anyhow!(
                "failed to spawn MCP server command '{}' for {}: {e}",
                server.command,
                server.name
            )
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("failed to open MCP server stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("failed to open MCP server stdout"))?;

        let mut session = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };

        session
            .request(
                "initialize",
                json!({
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": {
                        "name": "workclaw-runtime",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                }),
            )
            .await?;
        session
            .notify("notifications/initialized", json!({}))
            .await?;
        Ok(session)
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.write_message(request).await?;
        self.read_response(id).await
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        self.write_message(json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
        .await
    }

    async fn write_message(&mut self, message: Value) -> Result<()> {
        let body = serde_json::to_vec(&message)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        timeout(MCP_TIMEOUT, self.stdin.write_all(header.as_bytes()))
            .await
            .map_err(|_| anyhow!("timed out writing MCP JSON-RPC header"))??;
        timeout(MCP_TIMEOUT, self.stdin.write_all(&body))
            .await
            .map_err(|_| anyhow!("timed out writing MCP JSON-RPC body"))??;
        timeout(MCP_TIMEOUT, self.stdin.flush())
            .await
            .map_err(|_| anyhow!("timed out flushing MCP JSON-RPC request"))??;
        Ok(())
    }

    async fn read_response(&mut self, id: u64) -> Result<Value> {
        loop {
            let message = self.read_message(id).await?;
            if message.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = message.get("error") {
                return Err(anyhow!("{error}"));
            }
            return message
                .get("result")
                .cloned()
                .ok_or_else(|| anyhow!("missing JSON-RPC result for response id {id}"));
        }
    }

    async fn read_message(&mut self, id: u64) -> Result<Value> {
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = Vec::new();
            let bytes = timeout(MCP_TIMEOUT, self.stdout.read_until(b'\n', &mut line))
                .await
                .map_err(|_| anyhow!("timed out waiting for MCP header for response id {id}"))??;
            if bytes == 0 {
                return Err(anyhow!("MCP server closed stdout before response id {id}"));
            }
            let line = String::from_utf8_lossy(&line);
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break;
            }
            if let Some((name, value)) = trimmed.split_once(':') {
                if name.eq_ignore_ascii_case("content-length") {
                    content_length = Some(value.trim().parse::<usize>().map_err(|e| {
                        anyhow!("invalid MCP Content-Length for response id {id}: {e}")
                    })?);
                }
            }
        }

        let length = content_length
            .ok_or_else(|| anyhow!("missing MCP Content-Length for response id {id}"))?;
        let mut body = vec![0_u8; length];
        timeout(MCP_TIMEOUT, self.stdout.read_exact(&mut body))
            .await
            .map_err(|_| anyhow!("timed out reading MCP body for response id {id}"))??;
        serde_json::from_slice(&body)
            .map_err(|e| anyhow!("failed to parse MCP JSON-RPC response for id {id}: {e}"))
    }

    async fn close(&mut self) {
        let _ = self.stdin.shutdown().await;
        if timeout(Duration::from_secs(1), self.child.wait())
            .await
            .is_err()
        {
            let _ = self.child.kill().await;
            let _ = self.child.wait().await;
        }
    }
}
