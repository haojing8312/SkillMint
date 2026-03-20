# Feishu Settings Console Design

**Date:** 2026-03-19

**Goal**

Refactor the existing `渠道连接器 > 飞书` settings page into a clearer Feishu console that keeps all Feishu-related workflows in one place while exposing the new OpenClaw official plugin compatibility features.

## Product Decision

Use the existing settings page as the foundation, but reorganize the Feishu area into second-level sections instead of continuing to append more cards to one long page.

This keeps the user in the same “I am configuring Feishu” context while making the page scalable enough for:

- base connection settings
- official plugin host visibility
- pairing and authorization operations

## Recommended Information Architecture

Keep the top-level tab unchanged:

- `渠道连接器 > 飞书`

Inside the Feishu panel, introduce three second-level sections:

1. `连接配置`
2. `官方插件`
3. `配对与授权`

This is the best UX balance because:

- it avoids splitting Feishu into multiple pages
- it avoids one extremely long form
- it matches the real user journey from “connect” to “understand plugin state” to “approve access”

## Section Design

### 1. 连接配置

Purpose: help the user answer “Feishu 有没有接通”.

Content:

- app credentials
- callback / verification settings
- sidecar or connector base URL
- current connection status
- diagnostics and retry entry

What stays here:

- existing App ID / App Secret inputs
- retry / reconnect actions
- connector diagnostics summary

What moves out:

- official plugin host summary
- official plugin account snapshots
- pairing requests

### 2. 官方插件

Purpose: help the user answer “官方插件现在在 WorkClaw 里是否可用，以及它如何理解当前配置”.

Content:

- official plugin installed/not installed
- plugin package/version
- channel host summary
- default account
- account snapshots returned by the official plugin
- capability badges
- warnings from official plugin security checks

Design rule:

This section should present the official plugin as the primary compatibility path, not as an obscure advanced setting. The user should understand that WorkClaw is hosting the official Feishu plugin.

### 3. 配对与授权

Purpose: help the user answer “谁正在申请访问机器人，以及我如何批准/拒绝”.

Content:

- pending pairing requests
- approved / denied history
- request code
- sender ID
- account ID
- request creation time
- approve / deny actions
- effective paired allowlist preview

Design rule:

This section should feel operational, not technical. The primary action should be reviewing requests, not editing raw storage.

## UX Flow

### First-time setup

1. User enters Feishu connection settings in `连接配置`.
2. User sees official plugin host/account status in `官方插件`.
3. User receives first DM pairing request in `配对与授权`.
4. User approves request.
5. Future messages from that sender are allowed automatically.

### Ongoing operations

The user returns to the same Feishu page for:

- diagnosing connection issues
- checking whether the official plugin is healthy
- approving new pairing requests

## Visual Hierarchy

Recommended structure:

- Feishu page header
- compact status strip
- second-level section tabs
- one focused content panel below

The status strip should show:

- connection status
- official plugin status
- default account
- pending pairing count

This gives the user immediate orientation without requiring scrolling.

## Data Model Mapping

### 连接配置

Data sources:

- existing `get_feishu_gateway_settings`
- existing connector diagnostics/status commands

### 官方插件

Data sources:

- existing `list_openclaw_plugin_channel_hosts`
- existing `get_openclaw_plugin_feishu_channel_snapshot`

### 配对与授权

Data sources:

- new `list_feishu_pairing_requests`
- new `approve_feishu_pairing_request`
- new `deny_feishu_pairing_request`

## Behavior Rules

### Loading

- Load the Feishu page once, then populate each section independently.
- Do not block the whole page on one failing subsection.
- Show partial success when base connection works but plugin snapshot fails, or when plugin works but pairing data is empty.

### Errors

- `连接配置` errors should remain inline and actionable.
- `官方插件` errors should be framed as compatibility-host status issues.
- `配对与授权` errors should preserve the request list if already loaded.

### Empty states

- No official plugin installed: show a guided compatibility message.
- No pairing requests: show “暂无配对申请”.
- No configured default account: show warning but do not hide the page.

## Why This Beats The Alternatives

### Better than keeping one long page

- lower cognitive load
- easier scanning
- more scalable for future plugin features

### Better than opening a separate plugin page

- preserves user mental model
- avoids splitting one Feishu workflow across multiple places
- keeps support and troubleshooting simpler

## Phase Delivery Recommendation

Implement in two passes:

### Pass 1

Restructure the Feishu page into second-level sections and move existing host/snapshot content into `官方插件`.

### Pass 2

Add the `配对与授权` section with real approve/deny actions and pending count in the status strip.

This keeps the UI change incremental while immediately improving clarity.

## Acceptance Criteria

- The user can configure Feishu without seeing an overloaded page.
- The user can clearly tell whether the official plugin host is running and how it resolved accounts.
- The user can review and resolve pairing requests from the same Feishu page.
- The page still feels like one Feishu configuration surface, not three unrelated tools.
