# Skill Invoke Display Name Mapping Design

**Problem**

`skill` 工具要求 `skill_name` 使用 ASCII 目录名/内部标识符，但运行时注入给模型的 `<available_skills>` 只暴露了展示名和 `SKILL.md` 路径。模型因此可能把中文展示名直接填进 `skill_name`，触发 `INVALID_SKILL_NAME`。

**Approved Direction**

1. 主修提示词契约：
   在 `<available_skills>` 中显式暴露可调用标识，并明确说明 `skill` 工具必须使用该标识或 `SKILL.md` 路径，不应使用展示名。
2. 辅修工具兜底：
   `skill` 工具在收到展示名时，尝试在可用技能根目录中按 `SKILL.md` frontmatter `name` 做一次映射，再回落到现有目录名解析。

**Why This Approach**

- 直接修正模型可见契约，减少错误生成。
- 给运行时增加容错，覆盖旧 prompt、模型漂移和人工输入场景。
- 改动集中在已有技能提示词与 `skill` 工具，不扩散到会话/前端主流程。

**Scope**

- 修改工作区技能 prompt 的结构。
- 修改 `skill` 工具的解析逻辑。
- 补充回归测试，覆盖 prompt 输出和展示名调用。

**Out of Scope**

- 改动技能安装数据结构。
- 改动前端执行记录展示。
- 大规模重构技能发现机制。
