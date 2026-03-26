use runtime_lib::agent::tools::search_providers::cache::SearchCache;
use runtime_lib::agent::tools::search_providers::{
    SearchItem, SearchParams, SearchProvider, SearchResponse,
};
use runtime_lib::agent::tools::WebSearchTool;
use runtime_lib::agent::{Tool, ToolContext};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 用于测试的 Mock Provider
struct MockProvider;

impl SearchProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }
    fn display_name(&self) -> &str {
        "Mock Search"
    }
    fn search(&self, params: &SearchParams) -> anyhow::Result<SearchResponse> {
        Ok(SearchResponse {
            query: params.query.clone(),
            provider: "mock".to_string(),
            items: vec![SearchItem {
                title: "Mock Result".to_string(),
                url: "https://example.com".to_string(),
                snippet: "A mock search result".to_string(),
            }],
            elapsed_ms: 1,
        })
    }
}

struct RecordingProvider {
    last_query: Arc<Mutex<Option<String>>>,
    last_freshness: Arc<Mutex<Option<String>>>,
}

impl SearchProvider for RecordingProvider {
    fn name(&self) -> &str {
        "recording"
    }

    fn display_name(&self) -> &str {
        "Recording Search"
    }

    fn search(&self, params: &SearchParams) -> anyhow::Result<SearchResponse> {
        *self.last_query.lock().expect("query lock") = Some(params.query.clone());
        *self.last_freshness.lock().expect("freshness lock") = params.freshness.clone();
        Ok(SearchResponse {
            query: params.query.clone(),
            provider: "recording".to_string(),
            items: vec![SearchItem {
                title: "Recorded Result".to_string(),
                url: "https://example.com/recorded".to_string(),
                snippet: "Recorded request".to_string(),
            }],
            elapsed_ms: 1,
        })
    }
}

fn make_tool() -> WebSearchTool {
    let cache = Arc::new(SearchCache::new(Duration::from_secs(60), 10));
    WebSearchTool::with_provider(Box::new(MockProvider), cache)
}

#[test]
fn test_web_search_tool_metadata() {
    let tool = make_tool();
    assert_eq!(tool.name(), "web_search");
    assert!(!tool.description().is_empty());

    let schema = tool.input_schema();
    assert!(schema["properties"]["query"].is_object());
    assert!(schema["required"]
        .as_array()
        .unwrap()
        .contains(&json!("query")));
}

#[test]
fn test_web_search_missing_query() {
    let tool = make_tool();
    let ctx = ToolContext::default();
    let result = tool.execute(json!({}), &ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("query"));
}

#[test]
fn test_web_search_empty_query() {
    let tool = make_tool();
    let ctx = ToolContext::default();
    let result = tool.execute(json!({"query": "  "}), &ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("不能为空"));
}

#[test]
fn test_web_search_with_mock_provider() {
    let tool = make_tool();
    let ctx = ToolContext::default();
    let result = tool.execute(json!({"query": "test search"}), &ctx);
    assert!(result.is_ok(), "搜索失败: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("已使用搜索引擎：Mock Search"));
    assert!(output.contains("Mock Result"));
    assert!(output.contains("example.com"));
}

#[test]
fn test_web_search_cache_hit() {
    let cache = Arc::new(SearchCache::new(Duration::from_secs(60), 10));
    let tool = WebSearchTool::with_provider(Box::new(MockProvider), Arc::clone(&cache));
    let ctx = ToolContext::default();

    // 第一次搜索
    let result1 = tool.execute(json!({"query": "cached query"}), &ctx);
    assert!(result1.is_ok());

    // 第二次搜索同样的 query，应该命中缓存
    let result2 = tool.execute(json!({"query": "cached query"}), &ctx);
    assert!(result2.is_ok());
    assert_eq!(result1.unwrap(), result2.unwrap());
}

#[test]
fn test_web_search_normalizes_today_queries() {
    let last_query = Arc::new(Mutex::new(None));
    let last_freshness = Arc::new(Mutex::new(None));
    let tool = WebSearchTool::with_provider(
        Box::new(RecordingProvider {
            last_query: Arc::clone(&last_query),
            last_freshness: Arc::clone(&last_freshness),
        }),
        Arc::new(SearchCache::new(Duration::from_secs(60), 10)),
    );

    let result = tool.execute(
        json!({"query": "帮我搜一下今天的 AI 新闻，并给我一个简报"}),
        &ToolContext::default(),
    );

    assert!(result.is_ok(), "搜索失败: {:?}", result);
    let query = last_query
        .lock()
        .expect("query lock")
        .clone()
        .expect("query should be recorded");
    let freshness = last_freshness.lock().expect("freshness lock").clone();
    assert!(query.contains("AI 新闻"));
    assert!(!query.contains("今天"));
    assert!(query.contains("20"));
    assert_eq!(freshness.as_deref(), Some("day"));
}

#[test]
fn test_web_search_normalizes_this_month_queries() {
    let last_query = Arc::new(Mutex::new(None));
    let last_freshness = Arc::new(Mutex::new(None));
    let tool = WebSearchTool::with_provider(
        Box::new(RecordingProvider {
            last_query: Arc::clone(&last_query),
            last_freshness: Arc::clone(&last_freshness),
        }),
        Arc::new(SearchCache::new(Duration::from_secs(60), 10)),
    );

    let result = tool.execute(
        json!({"query": "整理一下这个月的 AI 融资新闻"}),
        &ToolContext::default(),
    );

    assert!(result.is_ok(), "搜索失败: {:?}", result);
    let query = last_query
        .lock()
        .expect("query lock")
        .clone()
        .expect("query should be recorded");
    let freshness = last_freshness.lock().expect("freshness lock").clone();
    assert!(query.contains("AI 融资新闻"));
    assert!(!query.contains("这个月"));
    assert!(query.contains("年"));
    assert!(query.contains("月"));
    assert_eq!(freshness.as_deref(), Some("month"));
}

#[test]
fn test_create_all_known_providers() {
    use runtime_lib::agent::tools::search_providers::create_provider;

    let formats = [
        "search_brave",
        "search_tavily",
        "search_metaso",
        "search_bocha",
        "search_serpapi",
    ];
    for fmt in &formats {
        let result = create_provider(fmt, "https://example.com", "test_key", "google");
        assert!(result.is_ok(), "Provider {} 创建失败", fmt);
    }
}

#[test]
fn test_create_unknown_provider() {
    use runtime_lib::agent::tools::search_providers::create_provider;
    let result = create_provider("search_unknown", "", "key", "");
    assert!(result.is_err());
}
