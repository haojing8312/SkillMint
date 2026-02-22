use serde::{Deserialize, Serialize};

/// Agent 工具执行权限模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PermissionMode {
    /// 默认：Write/Edit/Bash 需要用户确认
    Default,
    /// 接受编辑：Write/Edit 自动通过，Bash 仍需确认
    AcceptEdits,
    /// 无限制：所有工具自动通过
    Unrestricted,
}

impl PermissionMode {
    /// 判断指定工具是否需要用户确认
    pub fn needs_confirmation(&self, tool_name: &str) -> bool {
        match self {
            Self::Unrestricted => false,
            Self::AcceptEdits => matches!(tool_name, "bash"),
            Self::Default => matches!(tool_name, "write_file" | "edit" | "bash"),
        }
    }
}

impl Default for PermissionMode {
    fn default() -> Self {
        Self::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode_is_default_variant() {
        // 确认 Default::default() 返回 Default 变体
        assert_eq!(PermissionMode::default(), PermissionMode::Default);
    }
}
