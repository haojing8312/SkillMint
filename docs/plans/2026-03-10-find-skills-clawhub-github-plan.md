# Find Skills ClawHub And GitHub Fallback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 WorkClaw“找技能”与 ClawHub 网页搜索结果不一致的问题，并支持 GitHub 仓库下载到工作空间后的一键本地导入。

**Architecture:** 统一 ClawHub 搜索到网页同源的 `/api/v1/skills` 列表接口，保留相关性排序但移除低置信度回退。对 GitHub fallback，新增后端仓库下载与技能目录扫描/批量导入命令，前端在聊天安装卡片中支持 GitHub 仓库候选的一键导入。

**Tech Stack:** Rust + Tauri commands, React + Vitest, existing local-skill import pipeline.

---

### Task 1: 修复 ClawHub 搜索接口
- 为 ClawHub 搜索补失败测试
- 切换搜索实现到网页同源列表接口
- 保留推荐排序，过滤掉无关键词命中的候选

### Task 2: 增加 GitHub 仓库导入能力
- 为技能目录扫描补失败测试
- 增加 GitHub 仓库下载到工作空间并发现多个技能目录的命令
- 单技能自动导入，多技能仓库批量导入并返回结果

### Task 3: 接入聊天安装卡片
- 为 GitHub 仓库安装卡片补前端测试
- 让聊天卡片支持 GitHub 仓库候选
- 完成安装确认、批量导入和结果刷新

### Task 4: 验证
- 运行相关 Rust 测试
- 运行相关前端测试
