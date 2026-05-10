use crate::agent::tool_manifest::{ToolCategory, ToolMetadata, ToolSource};
use crate::agent::tools::tool_result;
use crate::agent::{Tool, ToolContext};
use crate::commands::chat_media_store::read_inbound_media_ref;
use crate::runtime_paths::RuntimePaths;
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;

const DEFAULT_CHUNK_CHARS: usize = 12_000;
const MIN_CHUNK_CHARS: usize = 1_000;
const MAX_CHUNK_CHARS: usize = 50_000;
const MAX_DOCUMENT_BYTES: usize = 25 * 1024 * 1024;

pub struct DocumentAnalyzeTool {
    runtime_paths: RuntimePaths,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DocumentTextChunk {
    index: usize,
    char_start: usize,
    char_end: usize,
    char_count: usize,
    text: String,
}

impl DocumentAnalyzeTool {
    pub(crate) fn new(runtime_paths: RuntimePaths) -> Self {
        Self { runtime_paths }
    }

    pub fn with_runtime_root(runtime_root: impl Into<PathBuf>) -> Self {
        Self::new(RuntimePaths::new(runtime_root))
    }

    fn read_text(&self, media_ref: &str) -> Result<String> {
        let bytes = read_inbound_media_ref(&self.runtime_paths, media_ref, MAX_DOCUMENT_BYTES)
            .map_err(|err| anyhow!(err))?;
        String::from_utf8(bytes)
            .map_err(|err| anyhow!("媒体引用 {media_ref} 不是 UTF-8 文本: {err}"))
    }
}

impl Tool for DocumentAnalyzeTool {
    fn name(&self) -> &str {
        "document_analyze"
    }

    fn description(&self) -> &str {
        "Read a claim-checked large text, Markdown, or PDF-extracted document by mediaRef and return bounded chunks for full-document analysis. Use this when the user asks to analyze, summarize, extract sections, or answer questions from the entire attached document instead of relying only on the preview."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "mediaRef": {
                    "type": "string",
                    "description": "Claim-check reference such as media://inbound/<id>"
                },
                "mimeType": {
                    "type": "string",
                    "description": "Original document MIME type, for example text/markdown or application/pdf"
                },
                "analysisGoal": {
                    "type": "string",
                    "enum": ["summarize", "extract_structure", "answer_question"],
                    "description": "How the caller intends to use the returned chunks"
                },
                "question": {
                    "type": "string",
                    "description": "Optional user question when analysisGoal is answer_question"
                },
                "chunkChars": {
                    "type": "integer",
                    "minimum": 1000,
                    "maximum": 50000,
                    "default": DEFAULT_CHUNK_CHARS
                }
            },
            "required": ["mediaRef", "analysisGoal"]
        })
    }

    fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<String> {
        let media_ref = input
            .get("mediaRef")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("mediaRef is required"))?;
        let analysis_goal = input
            .get("analysisGoal")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("analysisGoal is required"))?;
        let chunk_chars = input
            .get("chunkChars")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_CHUNK_CHARS)
            .clamp(MIN_CHUNK_CHARS, MAX_CHUNK_CHARS);
        let mime_type = input
            .get("mimeType")
            .and_then(Value::as_str)
            .unwrap_or("text/plain");
        let question = input.get("question").and_then(Value::as_str);

        let text = self.read_text(media_ref)?;
        let total_chars = text.chars().count();
        let chunks = split_text_chunks(&text, chunk_chars);
        let chunk_count = chunks.len();

        tool_result::success(
            self.name(),
            format!("已读取完整文档并切分为 {chunk_count} 个块"),
            json!({
                "status": "ok",
                "mediaRef": media_ref,
                "mimeType": mime_type,
                "analysisGoal": analysis_goal,
                "question": question,
                "chunkChars": chunk_chars,
                "chunkCount": chunk_count,
                "totalChars": total_chars,
                "truncated": false,
                "analysis": "document chunks returned for caller-side synthesis",
                "chunks": chunks,
            }),
        )
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            display_name: Some("Document Analyze".to_string()),
            category: ToolCategory::File,
            read_only: true,
            destructive: false,
            concurrency_safe: true,
            open_world: false,
            requires_approval: false,
            source: ToolSource::Runtime,
        }
    }
}

fn split_text_chunks(text: &str, chunk_chars: usize) -> Vec<DocumentTextChunk> {
    let chunk_chars = chunk_chars.clamp(MIN_CHUNK_CHARS, MAX_CHUNK_CHARS);
    let mut chunks = Vec::new();
    let mut chunk_start_byte = 0;
    let mut chunk_start_char = 0;
    let mut chars_in_chunk = 0;

    for (byte_index, _) in text.char_indices() {
        if chars_in_chunk == chunk_chars {
            chunks.push(DocumentTextChunk {
                index: chunks.len(),
                char_start: chunk_start_char,
                char_end: chunk_start_char + chars_in_chunk,
                char_count: chars_in_chunk,
                text: text[chunk_start_byte..byte_index].to_string(),
            });
            chunk_start_byte = byte_index;
            chunk_start_char += chars_in_chunk;
            chars_in_chunk = 0;
        }
        chars_in_chunk += 1;
    }

    if chars_in_chunk > 0 {
        chunks.push(DocumentTextChunk {
            index: chunks.len(),
            char_start: chunk_start_char,
            char_end: chunk_start_char + chars_in_chunk,
            char_count: chars_in_chunk,
            text: text[chunk_start_byte..].to_string(),
        });
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::chat_media_store::save_inbound_media;
    use serde_json::{json, Value};
    use tempfile::tempdir;

    #[test]
    fn document_analyze_rejects_unsafe_media_refs() {
        let temp = tempdir().expect("tempdir");
        let runtime_paths = RuntimePaths::new(temp.path().join("runtime-root"));
        let tool = DocumentAnalyzeTool::new(runtime_paths);

        for media_ref in [
            "media://inbound/../evil.md",
            "media://other/id",
            "media://inbound/a/b.md",
        ] {
            let error = tool
                .execute(
                    json!({
                        "mediaRef": media_ref,
                        "analysisGoal": "summarize",
                    }),
                    &ToolContext::default(),
                )
                .expect_err("unsafe ref should fail");
            assert!(
                error.to_string().contains("媒体引用"),
                "unexpected error for {media_ref}: {error}"
            );
        }
    }

    #[test]
    fn document_analyze_chunks_saved_utf8_text_on_char_boundaries() {
        let temp = tempdir().expect("tempdir");
        let runtime_paths = RuntimePaths::new(temp.path().join("runtime-root"));
        let text = format!(
            "{}{}{}",
            "甲".repeat(1000),
            "乙".repeat(1000),
            "丙".repeat(500)
        );
        let saved =
            save_inbound_media(&runtime_paths, text.as_bytes(), "text/markdown", "large.md")
                .expect("save media");
        let tool = DocumentAnalyzeTool::new(runtime_paths);

        let output = tool
            .execute(
                json!({
                    "mediaRef": saved.media_ref,
                    "mimeType": "text/markdown",
                    "analysisGoal": "extract_structure",
                    "chunkChars": 1000,
                }),
                &ToolContext::default(),
            )
            .expect("tool output");
        let parsed: Value = serde_json::from_str(&output).expect("json output");
        let details = &parsed["details"];

        assert_eq!(details["chunkCount"].as_u64(), Some(3));
        assert_eq!(details["totalChars"].as_u64(), Some(2500));
        assert_eq!(details["chunks"][0]["char_start"].as_u64(), Some(0));
        assert_eq!(details["chunks"][0]["char_end"].as_u64(), Some(1000));
        assert_eq!(
            details["chunks"][0]["text"].as_str().expect("chunk text"),
            "甲".repeat(1000)
        );
        assert_eq!(details["chunks"][2]["char_start"].as_u64(), Some(2000));
        assert_eq!(details["chunks"][2]["char_end"].as_u64(), Some(2500));
    }
}
