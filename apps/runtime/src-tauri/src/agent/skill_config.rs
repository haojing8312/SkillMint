use serde::Deserialize;

/// 从 SKILL.md 解析出的 Skill 配置
#[derive(Debug, Clone, Default)]
pub struct SkillConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub max_iterations: Option<usize>,
    pub system_prompt: String,
}

/// YAML frontmatter 的反序列化结构
#[derive(Deserialize, Default)]
struct FrontMatter {
    name: Option<String>,
    description: Option<String>,
    allowed_tools: Option<Vec<String>>,
    model: Option<String>,
    max_iterations: Option<usize>,
}

impl SkillConfig {
    /// 解析 SKILL.md 内容，提取 YAML frontmatter 和 system prompt
    pub fn parse(content: &str) -> Self {
        // 没有 frontmatter 标记
        if !content.starts_with("---") {
            return Self {
                system_prompt: content.to_string(),
                ..Default::default()
            };
        }

        // 跳过开头的 "---\n"，查找第二个 "---"
        let rest = &content[3..];
        let end_pos = match rest.find("\n---") {
            Some(pos) => pos,
            None => {
                // 没找到结束标记，整个内容作为 prompt
                return Self {
                    system_prompt: content.to_string(),
                    ..Default::default()
                };
            }
        };

        let yaml_str = &rest[..end_pos];
        // "---" (3) + yaml + "\n---" (4) = prompt 开始位置
        let prompt_start = 3 + end_pos + 4;
        let system_prompt = if prompt_start < content.len() {
            content[prompt_start..].trim_start_matches('\n').to_string()
        } else {
            String::new()
        };

        let fm: FrontMatter = serde_yaml::from_str(yaml_str).unwrap_or_default();

        Self {
            name: fm.name,
            description: fm.description,
            allowed_tools: fm.allowed_tools,
            model: fm.model,
            max_iterations: fm.max_iterations,
            system_prompt,
        }
    }
}
