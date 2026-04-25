use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use crate::runtime_paths::RuntimePaths;

pub const CHAT_MEDIA_REF_PREFIX: &str = "media://inbound/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedChatMedia {
    pub media_ref: String,
    pub id: String,
    pub path: PathBuf,
    pub size_bytes: usize,
    pub mime_type: String,
}

pub fn chat_media_root(runtime_paths: &RuntimePaths) -> PathBuf {
    runtime_paths.cache_dir.join("chat-media").join("inbound")
}

pub fn save_inbound_media(
    runtime_paths: &RuntimePaths,
    bytes: &[u8],
    mime_type: &str,
    original_name: &str,
) -> Result<SavedChatMedia, String> {
    let root = chat_media_root(runtime_paths);
    fs::create_dir_all(&root)
        .map_err(|err| format!("创建媒体缓存目录失败 {}: {err}", root.display()))?;
    let normalized_mime = normalize_mime_type(mime_type);
    let id = build_media_id(original_name, &normalized_mime);
    let path = root.join(&id);
    fs::write(&path, bytes).map_err(|err| format!("写入媒体缓存失败 {}: {err}", path.display()))?;

    Ok(SavedChatMedia {
        media_ref: format!("{CHAT_MEDIA_REF_PREFIX}{id}"),
        id,
        path,
        size_bytes: bytes.len(),
        mime_type: normalized_mime,
    })
}

pub fn resolve_inbound_media_ref(
    runtime_paths: &RuntimePaths,
    media_ref: &str,
) -> Result<PathBuf, String> {
    let id = parse_inbound_media_id(media_ref)?;
    let root = chat_media_root(runtime_paths);
    let path = root.join(id);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|err| format!("媒体引用 {media_ref} 不存在或不可读: {err}"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("媒体引用 {media_ref} 指向符号链接，已拒绝"));
    }
    if !metadata.is_file() {
        return Err(format!("媒体引用 {media_ref} 不是文件"));
    }
    Ok(path)
}

pub fn read_inbound_media_ref(
    runtime_paths: &RuntimePaths,
    media_ref: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, String> {
    let path = resolve_inbound_media_ref(runtime_paths, media_ref)?;
    let metadata = fs::metadata(&path)
        .map_err(|err| format!("读取媒体元数据失败 {}: {err}", path.display()))?;
    if metadata.len() > max_bytes as u64 {
        return Err(format!("媒体引用 {media_ref} 超过 {max_bytes} 字节限制"));
    }
    fs::read(&path).map_err(|err| format!("读取媒体缓存失败 {}: {err}", path.display()))
}

pub fn delete_inbound_media_ref(
    runtime_paths: &RuntimePaths,
    media_ref: &str,
) -> Result<(), String> {
    let path = resolve_inbound_media_ref(runtime_paths, media_ref)?;
    fs::remove_file(&path).map_err(|err| format!("删除媒体缓存失败 {}: {err}", path.display()))
}

fn parse_inbound_media_id(media_ref: &str) -> Result<&str, String> {
    let Some(id) = media_ref.strip_prefix(CHAT_MEDIA_REF_PREFIX) else {
        return Err(format!("媒体引用 {media_ref} 不是 inbound media ref"));
    };
    if id.is_empty()
        || id == ".."
        || id.contains('/')
        || id.contains('\\')
        || id.contains('\0')
        || id.split('.').any(|part| part == "..")
    {
        return Err(format!("媒体引用 {media_ref} 包含不安全 ID"));
    }
    Ok(id)
}

fn build_media_id(original_name: &str, mime_type: &str) -> String {
    let stem = sanitize_filename_stem(original_name);
    let extension = media_extension(mime_type, original_name);
    if stem.is_empty() {
        format!("{}{}", Uuid::new_v4(), extension)
    } else {
        format!("{stem}---{}{}", Uuid::new_v4(), extension)
    }
}

fn normalize_mime_type(mime_type: &str) -> String {
    mime_type
        .split(';')
        .next()
        .unwrap_or("application/octet-stream")
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '+' | '.'))
        .collect::<String>()
        .if_empty("application/octet-stream")
}

fn sanitize_filename_stem(original_name: &str) -> String {
    let stem = original_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(original_name);
    stem.trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .chars()
        .take(48)
        .collect()
}

fn media_extension(mime_type: &str, original_name: &str) -> String {
    match mime_type {
        "image/jpeg" | "image/jpg" => ".jpg".to_string(),
        "image/png" => ".png".to_string(),
        "image/webp" => ".webp".to_string(),
        "image/gif" => ".gif".to_string(),
        "image/heic" => ".heic".to_string(),
        "image/heif" => ".heif".to_string(),
        _ => original_name
            .rsplit_once('.')
            .map(|(_, ext)| sanitize_extension(ext))
            .filter(|ext| !ext.is_empty())
            .map(|ext| format!(".{ext}"))
            .unwrap_or_else(|| ".bin".to_string()),
    }
}

fn sanitize_extension(extension: &str) -> String {
    extension
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(12)
        .collect()
}

trait IfEmpty {
    fn if_empty(self, fallback: &str) -> String;
}

impl IfEmpty for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::runtime_paths::RuntimePaths;

    use super::{
        CHAT_MEDIA_REF_PREFIX, read_inbound_media_ref, resolve_inbound_media_ref,
        save_inbound_media,
    };

    #[test]
    fn saves_and_reads_inbound_media_by_ref() {
        let temp = tempfile::tempdir().expect("tempdir");
        let runtime_paths = RuntimePaths::new(temp.path().join("runtime-root"));
        let saved = save_inbound_media(&runtime_paths, b"png-bytes", "image/png", "screen.png")
            .expect("save media");

        assert!(saved.media_ref.starts_with(CHAT_MEDIA_REF_PREFIX));
        assert!(
            saved
                .path
                .starts_with(runtime_paths.cache_dir.join("chat-media").join("inbound"))
        );
        assert!(saved.path.exists());
        assert_eq!(saved.size_bytes, "png-bytes".len());
        assert_eq!(saved.mime_type, "image/png");
        assert_eq!(
            read_inbound_media_ref(&runtime_paths, &saved.media_ref, 64).expect("read media"),
            b"png-bytes"
        );
    }

    #[test]
    fn rejects_unsafe_inbound_media_refs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let runtime_paths = RuntimePaths::new(temp.path().join("runtime-root"));

        for media_ref in [
            "media://inbound/",
            "media://other/file.png",
            "media://inbound/../evil.png",
            "media://inbound/a/b.png",
            "media://inbound/a\\b.png",
            "media://inbound/\0.png",
        ] {
            assert!(
                resolve_inbound_media_ref(&runtime_paths, media_ref).is_err(),
                "{media_ref:?} should be rejected"
            );
        }
    }

    #[test]
    fn read_rejects_oversized_media() {
        let temp = tempfile::tempdir().expect("tempdir");
        let runtime_paths = RuntimePaths::new(temp.path().join("runtime-root"));
        let saved = save_inbound_media(&runtime_paths, b"too-big", "image/png", "screen.png")
            .expect("save media");

        let error =
            read_inbound_media_ref(&runtime_paths, &saved.media_ref, 3).expect_err("too large");

        assert!(error.contains("超过"));
    }

    #[test]
    fn resolve_rejects_directories() {
        let temp = tempfile::tempdir().expect("tempdir");
        let runtime_paths = RuntimePaths::new(temp.path().join("runtime-root"));
        let dir = runtime_paths
            .cache_dir
            .join("chat-media")
            .join("inbound")
            .join("directory.png");
        fs::create_dir_all(&dir).expect("create directory");

        let error = resolve_inbound_media_ref(&runtime_paths, "media://inbound/directory.png")
            .expect_err("directory should be rejected");

        assert!(error.contains("不是文件"));
    }
}
