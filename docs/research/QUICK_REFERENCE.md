# Web Search Tools: Quick Reference Card

Keep this by your desk while implementing!

---

## Decision Trees

### Which Search API?

```
Do you need multiple search engines (Google + Baidu + Bing)?
├─ YES → SerpAPI ($75-275/mo)
└─ NO
    ├─ Budget conscious?
    │  ├─ YES → Brave Search ($3-5/1K searches)
    │  └─ NO → Tavily ($50/mo average)
    │
    ├─ Privacy-first?
    │  ├─ YES → DuckDuckGo (free)
    │  └─ NO → Tavily or Brave
    │
    └─ Building AI agent?
       ├─ YES → Tavily (has include_answer)
       └─ NO → SerpAPI or Google
```

### Which Content Extraction?

```
Is content mostly static HTML?
├─ YES → Trafilatura (free, 0.883 benchmark score)
└─ NO
    ├─ Has JavaScript rendering?
    │  ├─ YES → Firecrawl ($0.15-0.32/page) OR Playwright (free)
    │  └─ NO → Trafilatura (still best)
    │
    └─ Need structured JSON output?
       ├─ YES → Firecrawl (native)
       └─ NO → Trafilatura (Markdown is LLM-friendly)
```

### Caching Strategy?

```
How many searches per day?
├─ < 10/day → In-memory cache only (sufficient)
├─ 10-100/day → In-memory + SQLite cache
├─ 100-1000/day → + Semantic cache
└─ > 1000/day → + Redis + Prompt caching
```

---

## Pricing Cheat Sheet

### Per-Search Costs

| API | Cheapest | Average | Most Expensive |
|-----|----------|---------|----------------|
| **Tavily** | $0.003 (100K/mo) | $0.005 (10K/mo) | $0.012 (500/mo) |
| **Brave** | $0.0025 (100K+/mo) | $0.003 (10K/mo) | $0.005 (1K/mo) |
| **SerpAPI** | $0.0092 (30K/mo) | $0.015 (5K/mo) | $0.015 (overage) |
| **Google** | $0.005 CPM | $0.05 CPM | $∞ (unlimited) |
| **DuckDuckGo** | FREE | FREE | FREE |

### Extract Costs (per page)

| Tool | Cost | Format | Speed |
|------|------|--------|-------|
| **Trafilatura** | FREE | Markdown | ~500ms |
| **Firecrawl** | $0.15-0.32 | Markdown | ~1s |
| **Apify** | $0.20 CU/min | JSON | ~5-10s |
| **Oxylabs** | $49+ min | Varies | Enterprise |
| **Playwright** | FREE | HTML→MD | ~2-3s |

### Monthly Cost Examples

**For 10,000 searches/month**:
```
Tavily:              $50
Brave:               $30
SerpAPI:            $150
Self-hosted + Redis: $100
Tavily + Firecrawl:  $72
```

**For 100,000 searches/month** (with 70% caching):
```
Tavily + Cache:          $150 (30% hit rate)
Brave + Cache:           $90
SerpAPI + Cache:        $450
Tavily + Prompt Cache:   $50 (90% token reduction)
```

---

## API Parameters Quick Lookup

### Tavily Search

```python
# Minimum viable
tavily.search(query="machine learning")

# Production
tavily.search(
    query="latest AI frameworks",
    search_depth="advanced",        # 2 credits (slower, better)
    max_results=5,                  # 0-20
    include_answer=True,            # Include LLM summary
    include_raw_content="markdown", # or "html" or False
    chunks_per_source=2,            # 1-3 chunks
    topic="general",                # or "news"
    time_range="month",             # day/week/month/year
)

# Save tokens: use include_answer instead of processing yourself
```

### Brave Search

```python
brave.search(
    q="query string",                    # URL encoded
    count=10,                            # 1-20 results
    search_lang="en",                    # Language
    ui_lang="en",                        # UI language
    spellcheck=True,                     # Spell correction
    freshness="pd" | "pw" | "pm" | "py" # past day/week/month/year
)
```

### SerpAPI

```python
serpapi.search(
    q="query",
    engine="google",              # google|bing|baidu|yandex|...
    location="United States",
    device="desktop",             # desktop|mobile|tablet
    num=10,                        # 1-100 results
    start=0,                       # Pagination
)
```

### Trafilatura Extract

```python
trafilatura.extract(
    html,
    output_format="markdown",     # markdown|json|html|txt|xml|csv|xmltei
    with_metadata=True,
    include_formatting=True,
    include_links=True,
    include_images=False,
    favor_precision=True,         # vs favor_recall=True
)
```

---

## Token Count Estimation

### Tokens per search result (Claude 3.5 Sonnet)

```
HTML (raw):              3,000 tokens / page
HTML (cleaned):          2,500 tokens / page
Markdown:                  500 tokens / page
Markdown (concise):        200 tokens / page
Tavily summary:            100 tokens / page

5 search results:
- HTML:      15,000 tokens = $0.045 (input tokens)
- Markdown:   2,500 tokens = $0.0075
- Summary:      500 tokens = $0.0015
```

### Token budget allocation

Assume 2M context window, use 10% for search results:
```
200K token budget for search
÷ 5 results per search
= 40K tokens per result
= ~100 pages of markdown
```

**Practical limit**: 5-10 results @ 500-2000 tokens each = 2500-20000 tokens per query

---

## Common Use Cases

### Use Case 1: "Quick answer" Bot
```
Search: Tavily (basic) + include_answer: true
Cost: $0.005 per query
Latency: ~500ms
Tokens: ~1000 (just summary)
```

### Use Case 2: "Comprehensive research" Agent
```
Search: Tavily (advanced)
Extract: Top 3 URLs with Trafilatura
Cost: $0.010 + ($0 × 3)
Latency: ~1500ms + 1500ms
Tokens: ~3000 (full content)
```

### Use Case 3: "Multi-perspective" Agent
```
Search 1: Tavily
Search 2: Brave (different ranking)
Fallback: DuckDuckGo (if both fail)
Cost: $0.005 + $0.003 + $0 = $0.008
Latency: ~2 seconds
Tokens: ~2000 (deduplicated)
```

### Use Case 4: "High-volume" Batch Processor
```
Cache: Semantic (85% hit rate)
Searches executed: 15% only
Cost: $0.005 × 0.15 = $0.00075 per query
Latency: Cache hit ~1ms
Tokens: Cached results (near-zero)
```

---

## Performance Benchmarks

### Content Extraction Quality

(From research paper comparisons)

| Tool | Precision | Recall | F1 Score | Speed |
|------|-----------|--------|----------|-------|
| **Trafilatura** | 0.91 | 0.87 | **0.883** | Fast |
| **Readability** | 0.85 | 0.80 | 0.825 | Fast |
| **jusText** | 0.80 | 0.75 | 0.775 | Slow |
| **Boilerpipe** | 0.78 | 0.70 | 0.740 | Medium |
| **Raw HTML** | 0.20 | 1.00 | 0.333 | Fast |

**Winner**: Trafilatura (highest F1 score)

### Search API Response Time

| API | P50 | P95 | P99 |
|-----|-----|-----|-----|
| **Brave** | 150ms | 400ms | 800ms |
| **Tavily (basic)** | 300ms | 600ms | 1200ms |
| **Tavily (advanced)** | 800ms | 1500ms | 2500ms |
| **SerpAPI** | 600ms | 1200ms | 2000ms |
| **Google API** | 500ms | 1000ms | 1800ms |

**Fastest**: Brave Search

---

## Cost Saving Tips

### Tip 1: Use include_answer
```
Without: Search ($0.005) + LLM summarization (2000 tokens = $0.006) = $0.011
With:    Search ($0.005) + tiny LLM usage ($0.001) = $0.006
Saves: ~45% on summarization tasks
```

### Tip 2: Enable Caching
```
Cache hit rate 70%:
Cost per query: $0.005 × 0.30 = $0.0015
Savings: 70% off
```

### Tip 3: Use Markdown Format
```
HTML: 3000 tokens × $0.003 = $0.009
MD:    500 tokens × $0.003 = $0.0015
Saves: 83% on token cost
```

### Tip 4: Semantic Caching
```
100 queries, 30 unique:
Actual executions: 30 × $0.005 = $0.15
Cached queries: 70 × $0 = $0
Cost per query: $0.0015
Saves: 70% off
```

### Tip 5: Batch Searches
```
Stagger 10 searches across 10 minutes
→ Improved cache hit rate from semantic overlap
→ Typical improvement: 10-20% additional savings
```

---

## Error Handling Quick Reference

### Common Errors & Fixes

| Error | Cause | Fix |
|-------|-------|-----|
| 401 Unauthorized | Wrong API key | Check credentials |
| 429 Too Many Requests | Rate limit | Add exponential backoff |
| 500 Internal Server | Provider issue | Retry with fallback |
| Timeout | Slow response | Increase timeout, use faster depth |
| Empty results | Bad query | Refine query, try fallback |
| Extraction failed | Invalid URL | Check URL, use alternative |

### Retry Strategy

```python
def search_with_retry(query, max_retries=3):
    for attempt in range(max_retries):
        try:
            return tavily.search(query)
        except Exception as e:
            wait = 2 ** attempt  # Exponential backoff
            if attempt == max_retries - 1:
                return brave.search(query)  # Fallback
            time.sleep(wait)
```

---

## Configuration Templates

### Minimum Viable Config

```json
{
  "search": {
    "primary": "tavily",
    "api_key": "your-key",
    "max_results": 5,
    "depth": "basic"
  },
  "cache": {
    "type": "sqlite",
    "ttl_days": 7
  }
}
```

### Production Config

```json
{
  "search": {
    "primary": "tavily",
    "fallback": "brave",
    "api_keys": {
      "tavily": "...",
      "brave": "..."
    },
    "config": {
      "max_results": 5,
      "depth": "advanced",
      "include_answer": true,
      "include_raw_content": "markdown",
      "chunks_per_source": 2
    }
  },
  "cache": {
    "layers": [
      {
        "type": "memory",
        "size": 10000
      },
      {
        "type": "sqlite",
        "path": "/data/search_cache.db",
        "ttl_days": 7
      },
      {
        "type": "semantic",
        "similarity_threshold": 0.85,
        "embedder": "all-MiniLM-L6-v2"
      }
    ]
  },
  "extraction": {
    "tool": "trafilatura",
    "format": "markdown",
    "include_metadata": true,
    "favor_precision": true
  },
  "browser": {
    "enabled": true,
    "tool": "playwright",
    "headless": true,
    "timeout_ms": 30000
  },
  "monitoring": {
    "track_costs": true,
    "track_latency": true,
    "track_cache_hit_rate": true
  }
}
```

---

## Monitoring Checklist

Track these metrics for optimization:

```
Daily:
- [ ] Total searches (should be decreasing with caching)
- [ ] Cache hit rate (target: 70%+)
- [ ] Avg response time (target: <1s with cache)
- [ ] Errors/retries (target: <1%)

Weekly:
- [ ] Total cost (track per provider)
- [ ] Cost per search (track trend)
- [ ] API usage by provider
- [ ] Top queries (identify caching opportunities)

Monthly:
- [ ] Total spend vs budget
- [ ] Cost optimization wins
- [ ] Provider performance comparison
- [ ] Update caching TTL if needed
```

---

## Provider API Endpoints

Quick reference for making direct calls:

| Provider | Endpoint | Auth |
|----------|----------|------|
| **Tavily** | `https://api.tavily.com/search` | API key in body |
| **Brave** | `https://api.search.brave.com/res/v1/web/search` | Token header |
| **SerpAPI** | `https://serpapi.com/search` | `api_key` param |
| **Google** | `https://cse.google.com/cse` | API key param |
| **DuckDuckGo** | `https://duckduckgo.com/` | None (no API) |

---

## Useful Libraries

### Python

```python
# Search
import tavily  # pip install tavily-python
from langchain_community.tools import TavilySearchResults

# Extraction
import trafilatura  # pip install trafilatura
from readability import Document  # pip install readability-lxml

# HTML to Markdown
from markdownify import markdownify as md  # pip install markdownify
import html2text  # pip install html2text

# Caching
from functools import lru_cache
import redis  # pip install redis

# Browser
from playwright.async_api import async_playwright  # pip install playwright

# Embeddings (semantic cache)
from sentence_transformers import SentenceTransformer  # pip install sentence-transformers
```

### Rust

```rust
// HTTP client
use reqwest::Client;

// Serialization
use serde::{Deserialize, Serialize};

// Database
use sqlx::SqlitePool;

// Async runtime
use tokio;

// Error handling
use anyhow::Result;
```

### JavaScript/TypeScript

```typescript
// Fetch API (built-in)
const response = await fetch(url);

// Extraction
import trafilatura from "trafilatura";  // npm install trafilatura

// Browser automation
import { chromium } from "playwright";  // npm install playwright

// Caching
import NodeCache from "node-cache";  // npm install node-cache
import Redis from "redis";            // npm install redis

// Embeddings
import { Embeddings } from "langchain/embeddings";
```

---

## Resources for Deeper Learning

### Papers & Research
- [BrowserGym Ecosystem](https://arxiv.org/abs/2412.05467) - ICLR 2025
- [OpenHands Platform](https://arxiv.org/abs/2407.16741) - ICLR 2025
- [SWE-agent](https://arxiv.org/abs/2405.15793) - NeurIPS 2024
- Content Extraction Benchmark (Chuniversiteit)

### Official Docs
- [Tavily API Reference](https://docs.tavily.com/)
- [LangChain Agents](https://python.langchain.com/docs/modules/agents/)
- [Trafilatura](https://trafilatura.readthedocs.io/)
- [Playwright](https://playwright.dev/)
- [Brave Search API](https://brave.com/search/api/)

### Blog Posts
- [FreeCodeCamp Web Search with Tavily](https://www.freecodecamp.org/news/how-to-add-real-time-web-search-to-your-llm-using-tavily/)
- [Token Limit Management](https://deepchecks.com/5-approaches-to-solve-llm-token-limits/)
- [HTML to Markdown Guide](https://glukhov.org/post/2025/10/convert-html-to-markdown-in-python/)

---

## Print This!

Save this page for offline reference while implementing.

**Last Updated**: February 24, 2026
**Status**: Ready for Use
