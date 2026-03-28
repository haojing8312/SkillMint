---
name: PPTX 演示助手
description: >
  生成、编辑和读取 PowerPoint 演示文稿。支持三类任务：
  1) 读取或分析现有 PPTX 文本内容；
  2) 在模板或已有演示文稿上做 XML 安全编辑；
  3) 从零创建新演示文稿并输出原生 .pptx 文件。
  适用于 .pptx 演示文稿创建、结构化改写、模板复用、讲稿梳理与版式优化。
metadata:
  openclaw:
    primaryEnv: python
    requires:
      bins:
        - node
      anyBins:
        - python
        - python3
        - py
---

# PPTX 演示助手

使用这个技能处理 `.pptx` 文件时，先按任务类型路由，再读取本目录中的对应参考文件。

## 路由

| 任务类型 | 做法 | 主要参考 |
|---|---|---|
| 读取/分析现有 PPTX | 用 `python -m markitdown presentation.pptx` 抽取文本 | `README.md` |
| 编辑现有模板或演示文稿 | 走 XML-safe 编辑流程：分析 → unpack → 改 slide XML → clean → pack | `skills/ppt-editing-skill/SKILL.md` |
| 从零创建新 PPTX | 先规划 deck 结构，再用 PptxGenJS 生成 slide JS 并编译 | `skills/ppt-orchestra-skill/SKILL.md` |

## 基本规则

1. 最终交付物必须是原生 `.pptx` 文件。
2. 如果用户已经给了模板或现成演示文稿，默认优先走“编辑”路线，不要重建整份文件。
3. 如果是从零生成，必须先确定页面结构，再开始逐页生成，避免整份 PPT 风格漂移。
4. 优先保证版式一致性、信息层级和可读性，不要为了塞更多内容破坏页面留白。

## 读取 / 分析

直接抽取文本：

```bash
python -m markitdown presentation.pptx
```

如果只是需要总结内容、提取大纲、检查占位文字或核对页面信息，这一步通常就够了。

## 编辑现有 PPTX

先阅读：

- `skills/ppt-editing-skill/SKILL.md`

编辑流程：

1. 复制用户提供的 `pptx` 为工作副本
2. 用 `markitdown` 分析结构
3. unpack 演示文稿
4. 完成结构变更：删除、复制、重排 slide
5. 修改 slide XML 内容
6. clean 清理孤儿资源
7. pack 回 `.pptx`

注意：

- 先做结构变更，再做逐页内容修改。
- 不要手工复制 slide 文件，优先复用目录内已有脚本/流程约束。
- 编辑时优先保留原主题、版式和视觉语言。

## 从零创建 PPTX

先阅读：

- `skills/ppt-orchestra-skill/SKILL.md`
- `skills/design-style-skill/SKILL.md`
- `skills/color-font-skill/SKILL.md`
- `skills/slide-making-skill/SKILL.md`

建议流程：

1. 明确用户目标、受众、页数范围和风格方向
2. 设计页面结构：封面、目录、章节分隔、内容页、总结页
3. 选择颜色和字体
4. 逐页生成 slide 模块
5. 编译输出最终 `.pptx`
6. 用 `markitdown` 回读文本做 QA

## 目录提示

- `README.md`: 总体工作流摘要
- `skills/ppt-editing-skill/SKILL.md`: 模板编辑和 XML 工作流
- `skills/ppt-orchestra-skill/SKILL.md`: 从零生成整份 deck 的编排方法
- `skills/design-style-skill/SKILL.md`: 风格与视觉约束
- `skills/color-font-skill/SKILL.md`: 配色与字体策略
- `skills/slide-making-skill/SKILL.md`: 单页 PptxGenJS 实现细节
- `agents/`: 多种页面类型的页面生成说明

## 输出要求

- 保持整份演示文稿的视觉一致性。
- 标记需要人工设计复核的页面。
- 交付前至少做一次文本抽取 QA，确认没有残留占位文字。
