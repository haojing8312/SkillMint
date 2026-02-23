/// 搜索结果 LRU + TTL 缓存
///
/// 缓存键格式：`"{provider}:{query_lowercase}:{count}"`
/// - TTL 到期的条目在下次访问时被清除
/// - 超出 max_size 时按插入顺序（最旧的）淘汰
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::SearchItem;

/// 单条缓存记录
struct CacheEntry {
    /// 缓存的搜索结果
    items: Vec<SearchItem>,
    /// 写入时刻
    created_at: Instant,
    /// 插入序号，用于 LRU 淘汰排序
    seq: u64,
}

/// 搜索结果缓存
pub struct SearchCache {
    /// 缓存存储
    store: HashMap<String, CacheEntry>,
    /// 缓存有效期
    ttl: Duration,
    /// 最大缓存条目数
    max_size: usize,
    /// 全局自增插入序号
    seq_counter: u64,
}

impl SearchCache {
    /// 创建缓存实例
    ///
    /// # 参数
    /// - `ttl`：缓存有效期
    /// - `max_size`：最大缓存条目数（超出时淘汰最旧条目）
    pub fn new(ttl: Duration, max_size: usize) -> Self {
        Self {
            store: HashMap::new(),
            ttl,
            max_size,
            seq_counter: 0,
        }
    }

    /// 构造缓存键
    fn make_key(provider: &str, query: &str, count: usize) -> String {
        format!("{}:{}:{}", provider, query.to_lowercase(), count)
    }

    /// 查询缓存，未命中或已过期则返回 None
    pub fn get(&mut self, provider: &str, query: &str, count: usize) -> Option<Vec<SearchItem>> {
        let key = Self::make_key(provider, query, count);
        if let Some(entry) = self.store.get(&key) {
            if entry.created_at.elapsed() < self.ttl {
                return Some(entry.items.clone());
            }
            // TTL 过期，移除条目
            self.store.remove(&key);
        }
        None
    }

    /// 写入缓存，若超出 max_size 则淘汰最旧条目
    pub fn put(&mut self, provider: &str, query: &str, count: usize, items: Vec<SearchItem>) {
        let key = Self::make_key(provider, query, count);

        // 若 key 已存在则先移除（后面重新插入以刷新 seq）
        self.store.remove(&key);

        // 超出容量时淘汰插入序号最小（最旧）的条目
        if self.store.len() >= self.max_size {
            if let Some(oldest_key) = self
                .store
                .iter()
                .min_by_key(|(_, e)| e.seq)
                .map(|(k, _)| k.clone())
            {
                self.store.remove(&oldest_key);
            }
        }

        self.seq_counter += 1;
        self.store.insert(
            key,
            CacheEntry {
                items,
                created_at: Instant::now(),
                seq: self.seq_counter,
            },
        );
    }

    /// 返回当前缓存条目数
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// 是否为空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造测试用 SearchItem 列表
    fn make_items(n: usize) -> Vec<SearchItem> {
        (0..n)
            .map(|i| SearchItem {
                title: format!("标题 {}", i),
                url: format!("https://example.com/{}", i),
                snippet: format!("摘要 {}", i),
            })
            .collect()
    }

    #[test]
    fn test_cache_hit() {
        // 写入后立即读取应命中缓存
        let mut cache = SearchCache::new(Duration::from_secs(60), 100);
        let items = make_items(3);
        cache.put("brave", "rust language", 5, items.clone());

        let result = cache.get("brave", "rust language", 5);
        assert!(result.is_some(), "应命中缓存");
        let cached = result.unwrap();
        assert_eq!(cached.len(), 3);
        assert_eq!(cached[0].title, "标题 0");
    }

    #[test]
    fn test_miss_different_query() {
        // 查询词不同，不应命中缓存
        let mut cache = SearchCache::new(Duration::from_secs(60), 100);
        cache.put("brave", "rust language", 5, make_items(3));

        let result = cache.get("brave", "python language", 5);
        assert!(result.is_none(), "不同查询词不应命中缓存");
    }

    #[test]
    fn test_miss_different_provider() {
        // Provider 不同，不应命中缓存
        let mut cache = SearchCache::new(Duration::from_secs(60), 100);
        cache.put("brave", "rust language", 5, make_items(3));

        let result = cache.get("tavily", "rust language", 5);
        assert!(result.is_none(), "不同 Provider 不应命中缓存");
    }

    #[test]
    fn test_expiry() {
        // TTL 为 1 纳秒，写入后应立即过期
        let mut cache = SearchCache::new(Duration::from_nanos(1), 100);
        cache.put("brave", "rust language", 5, make_items(2));

        // 等待缓存过期
        std::thread::sleep(Duration::from_millis(10));

        let result = cache.get("brave", "rust language", 5);
        assert!(result.is_none(), "TTL 过期后应返回 None");

        // 过期条目应已被清理
        assert_eq!(cache.len(), 0, "过期条目应被移除");
    }

    #[test]
    fn test_max_size_eviction() {
        // 容量为 2，第 3 次写入应淘汰最旧条目
        let mut cache = SearchCache::new(Duration::from_secs(60), 2);

        cache.put("brave", "query_a", 5, make_items(1));
        cache.put("brave", "query_b", 5, make_items(1));
        assert_eq!(cache.len(), 2);

        // 写入第 3 条，触发淘汰
        cache.put("brave", "query_c", 5, make_items(1));
        assert_eq!(cache.len(), 2, "超出容量时应淘汰最旧条目，保持 max_size");

        // query_a 是最旧的，应被淘汰
        assert!(
            cache.get("brave", "query_a", 5).is_none(),
            "最旧条目 query_a 应被淘汰"
        );

        // query_b 和 query_c 仍在缓存
        assert!(
            cache.get("brave", "query_b", 5).is_some(),
            "query_b 应仍在缓存"
        );
        assert!(
            cache.get("brave", "query_c", 5).is_some(),
            "query_c 应仍在缓存"
        );
    }

    #[test]
    fn test_case_insensitive() {
        // 查询词大小写不敏感
        let mut cache = SearchCache::new(Duration::from_secs(60), 100);
        cache.put("brave", "Rust Language", 5, make_items(2));

        // 小写查询应命中
        let result = cache.get("brave", "rust language", 5);
        assert!(result.is_some(), "查询词应大小写不敏感");

        // 全大写也应命中
        let result2 = cache.get("brave", "RUST LANGUAGE", 5);
        assert!(result2.is_some(), "全大写查询词也应命中缓存");
    }
}
