# OpenClaw Employee UX Redesign (Schema Merge + Conversational Config) Design

**Date:** 2026-03-03  
**Status:** Approved (User selected B + schema merge proposal)  
**Owner:** Runtime/IM/UX

---

## 1. Background

Current employee configuration exposes technical fields (`role_id`, `openclaw_agent_id`) that are hard for normal users to understand. The UI is a long form with mixed concerns, and OpenClaw multi-agent capability is only partially surfaced (routing is integrated, but `AGENTS.md`-driven authoring is not user-friendly).

The target direction is:

1. Keep OpenClaw-native capability (`AGENTS.md`, `SOUL.md`, `USER.md`)  
2. Merge user-facing identity fields into one friendly field (`员工编号`)  
3. Configure agent profile via conversational flow rather than manual markdown editing  
4. Improve skill naming hygiene by detecting duplicate display names at install/import time

---

## 2. Goals

1. Replace user-facing `role_id + openclaw_agent_id` with one field: `employee_id` (`员工编号`), auto-generated but editable.  
2. Preserve compatibility with existing runtime behavior by internally mirroring to legacy fields during migration period.  
3. Add conversational config flow to generate/update `AGENTS.md`, `SOUL.md`, `USER.md`.  
4. Reorganize employee UX into a 3-step flow: `基础信息 -> 飞书连接 -> 技能与智能体配置`.  
5. Add duplicate skill-name checks in install/import workflows.

---

## 3. Non-Goals

1. Full removal of legacy DB columns in this phase.  
2. Multi-IM provider expansion (still Feishu first).  
3. Runtime-level OpenClaw protocol changes outside current vendored boundary.

---

## 4. Product Decisions

### 4.1 Employee identity model

1. New primary user-facing identity: `employee_id` (label: `员工编号`).  
2. Auto-generated from name/role template (slug-like, collision-safe).  
3. Internal compatibility rule (phase 1):  
   - `role_id = employee_id`  
   - `openclaw_agent_id = employee_id`
4. Advanced override is hidden for normal UX (no extra expert toggle in phase 1).

### 4.2 Feishu credential wording

Rename:
1. `员工飞书 app_id` -> `机器人 App ID`  
2. `员工飞书 app_secret` -> `机器人 App Secret`

### 4.3 OpenClaw profile authoring

1. Add `对话配置智能体` entry for each employee.  
2. Wizard asks one question at a time and writes structured profile content.  
3. Output files:
   - `AGENTS.md`
   - `SOUL.md`
   - `USER.md`
4. Provide real-time preview and one-click apply.

### 4.4 Skill duplicate naming

At install/import:
1. Detect duplicate display `name` among installed skills.  
2. Block blind overwrite by default.  
3. Prompt user to rename or explicitly replace.

---

## 5. Architecture

### 5.1 Frontend

1. `EmployeeHubView` becomes step-based wizard container.  
2. New conversational component for OpenClaw profile generation.  
3. Existing Feishu routing wizard remains, but employee identity references `employee_id`.

### 5.2 Backend (Tauri)

1. Add `employee_id` to employee command DTOs.  
2. Maintain legacy field mirror in command layer.  
3. Add profile generation/apply commands:
   - generate profile draft from Q/A state
   - write markdown files to employee workspace
4. Add duplicate-skill-name validation in install/import code path.

### 5.3 Data layer (SQLite)

1. Add `employee_id` column + unique index.  
2. Migration backfill `employee_id` from `role_id` for existing data.  
3. Keep existing `role_id/openclaw_agent_id` columns during transition.

---

## 6. Data Flow

### 6.1 Employee save

1. UI submits `employee_id` + business fields.  
2. Tauri validates and normalizes ID.  
3. Command writes:
   - `employee_id` (new)
   - `role_id = employee_id` (compat)
   - `openclaw_agent_id = employee_id` (compat)

### 6.2 Conversational agent config

1. UI stores question/answer state locally.  
2. On preview, backend returns markdown drafts for 3 files.  
3. On apply, backend writes files into employee workspace path and returns paths/status.  
4. UI shows last applied timestamp + quick open links.

### 6.3 Skill install/import

1. Parse incoming manifest.  
2. Query installed skills by display name (case-insensitive).  
3. If conflict and no explicit replace, return conflict error payload.  
4. UI prompts rename/retry.

---

## 7. Error Handling

1. `employee_id` format invalid -> inline validation + blocked save.  
2. `employee_id` conflict -> deterministic error with suggested suffix.  
3. Profile file write failure -> partial-failure response per file and retry action.  
4. Duplicate skill name -> explicit conflict response (not generic DB error).  
5. Missing workspace path -> auto-resolve default and surface final location.

---

## 8. Testing Strategy

1. Rust tests:
   - employee_id migration/backfill and uniqueness
   - mirror rule (`role_id/openclaw_agent_id`)
   - profile draft/apply command behavior
   - duplicate skill name conflict in install/import
2. Frontend tests:
   - EmployeeHub 3-step flow
   - employee_id auto-generate + edit
   - conversational wizard preview/apply
   - duplicate-skill conflict UI prompt
3. Regression tests:
   - existing OpenClaw route resolution
   - Feishu dispatch/session mapping

---

## 9. Rollout Plan

1. Phase 1: soft merge + compatibility mirror + new UX entrypoints.  
2. Phase 2: migrate internal consumers to `employee_id` naming fully.  
3. Phase 3: evaluate safe removal of legacy columns/labels.

---

## 10. Acceptance Criteria

1. Employee UI no longer exposes `role_id`/`openclaw_agent_id`.  
2. Users can complete OpenClaw profile setup without editing markdown manually.  
3. Generated `AGENTS.md/SOUL.md/USER.md` are persisted and reusable.  
4. Skill duplicate names are detected before silent conflicts.  
5. Existing Feishu + OpenClaw routing regression suite remains green.

---

## 11. External References

1. OpenClaw Multi-Agent Concept: https://docs.openclaw.ai/zh-CN/concepts/multi-agent  
2. OpenClaw Agent Concept: https://docs.openclaw.ai/zh-CN/concepts/agent  
3. OpenClaw FAQ (workspace/files behavior): https://docs.openclaw.ai/zh-CN/help/faq

