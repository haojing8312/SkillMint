# OpenClaw Feishu Config Gap Checklist

**Reference source of truth:** [OpenClaw Feishu config schema](/d:/code/WorkClaw/references/openclaw/extensions/feishu/src/config-schema.ts#L155)

This checklist tracks which `channels.feishu` fields are already projected into WorkClaw, partially projected, or still missing from the WorkClaw compatibility host.

---

## Top-Level Channel State

- `enabled` — partial
- `defaultAccount` — implemented
- `appId` — implemented
- `appSecret` — implemented
- `encryptKey` — partial
- `verificationToken` — partial
- `domain` — implemented
- `connectionMode` — implemented
- `webhookPath` — implemented
- `webhookHost` — implemented
- `webhookPort` — implemented
- `capabilities` — missing
- `configWrites` — implemented

## DM And Allowlist Policy

- `dmPolicy` — partial
- `allowFrom` — implemented
- `dms.*.enabled` — missing
- `dms.*.systemPrompt` — missing

## Group Policy

- `groupPolicy` — implemented
- `groupAllowFrom` — implemented
- `groupSenderAllowFrom` — implemented
- `requireMention` — implemented
- `groups.*.requireMention` — partial
- `groups.*.tools` — implemented in backend projection
- `groups.*.skills` — implemented in backend projection
- `groups.*.enabled` — implemented in backend projection
- `groups.*.allowFrom` — implemented in backend projection
- `groups.*.systemPrompt` — implemented in backend projection
- `groups.*.groupSessionScope` — implemented in backend projection
- `groups.*.topicSessionMode` — implemented in backend projection
- `groups.*.replyInThread` — implemented in backend projection
- `groups.<groupId>.*` — implemented in backend projection via `feishu_groups` JSON; basic advanced-config UI added

## Session / Threading

- `historyLimit` — implemented in backend projection
- `dmHistoryLimit` — implemented in backend projection
- `dms.<id>.enabled` — implemented in backend projection; basic advanced-config UI added
- `dms.<id>.systemPrompt` — implemented in backend projection; basic advanced-config UI added
- `groupSessionScope` — implemented
- `topicSessionMode` — implemented
- `threadSession` — missing in OpenClaw schema naming, but equivalent behavior must be mapped through `groupSessionScope/topicSessionMode`
- `replyInThread` — implemented via advanced-config UI

## Reply Rendering

- `markdown.mode` — implemented via advanced-config UI
- `markdown.tableMode` — implemented via advanced-config UI
- `renderMode` — implemented via advanced-config UI
- `streaming` — implemented via advanced-config UI
- `blockStreamingCoalesce.enabled` — implemented in backend projection
- `blockStreamingCoalesce.minDelayMs` — implemented in backend projection
- `blockStreamingCoalesce.maxDelayMs` — implemented in backend projection
- `textChunkLimit` — implemented via advanced-config UI
- `chunkMode` — implemented via advanced-config UI
- `footer.*` — implemented in backend projection; basic advanced-config UI added

## Reactions / Typing / Presence

- `actions.reactions` — implemented
- `reactionNotifications` — implemented
- `typingIndicator` — implemented
- `heartbeat.visibility` — implemented in backend projection; advanced-config UI added
- `heartbeat.intervalMs` — implemented in backend projection; advanced-config UI added

## Media / HTTP / Transport

- `mediaMaxMb` — implemented in backend projection; advanced-config UI added
- `httpTimeoutMs` — implemented in backend projection; advanced-config UI added

## Tools / Permissions

- `tools.doc` — implemented
- `tools.chat` — implemented
- `tools.wiki` — implemented
- `tools.drive` — implemented
- `tools.perm` — implemented
- `tools.scopes` — implemented

## Sender Identity / Dynamic Agents

- `resolveSenderNames` — implemented
- `dynamicAgentCreation.enabled` — implemented in backend projection; advanced-config UI added
- `dynamicAgentCreation.workspaceTemplate` — implemented in backend projection; advanced-config UI added
- `dynamicAgentCreation.agentDirTemplate` — implemented in backend projection; advanced-config UI added
- `dynamicAgentCreation.maxAgents` — implemented in backend projection; advanced-config UI added

## Multi-Account

- `accounts.<id>.enabled` — implemented via backend inheritance
- `accounts.<id>.name` — implemented in backend projection
- `accounts.<id>.appId` — implemented
- `accounts.<id>.appSecret` — implemented
- `accounts.<id>.encryptKey` — implemented via backend inheritance
- `accounts.<id>.verificationToken` — implemented via backend inheritance
- `accounts.<id>.domain` — implemented
- `accounts.<id>.connectionMode` — implemented
- `accounts.<id>.webhookPath` — implemented in backend projection
- `accounts.<id>` inheritance of shared fields — implemented for current backend projection
- `accounts.<id>` per-account override source — implemented in backend projection via `feishu_account_overrides` JSON; basic advanced-config UI added

---

## Recommended Fill Order

1. `markdown.*`, `blockStreamingCoalesce.*`, `historyLimit`, `dmHistoryLimit`
2. `groups.*`, especially `tools/skills/systemPrompt/allowFrom`
3. multi-account full inheritance and `accounts.<id>.webhookPath`
4. webhook-only advanced fields and config writes UX
5. end-to-end validation for newly exposed advanced config fields

---

## Usage

When implementing a config field:

1. Add it to WorkClaw’s config projection in [openclaw_plugins.rs](/d:/code/WorkClaw/apps/runtime/src-tauri/src/commands/openclaw_plugins.rs)
2. Add the corresponding UI or advanced-config path in [SettingsView.tsx](/d:/code/WorkClaw/apps/runtime/src/components/SettingsView.tsx)
3. Add inspection/snapshot verification so the official plugin sees the projected value correctly
4. Move the field from `missing` or `partial` toward `implemented`
