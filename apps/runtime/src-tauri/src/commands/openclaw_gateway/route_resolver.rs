fn normalize_route_channel(raw: Option<&str>) -> String {
    let trimmed = raw.unwrap_or("app").trim();
    if trimmed.is_empty() {
        "app".to_string()
    } else {
        trimmed.to_lowercase()
    }
}

fn normalize_route_text(raw: Option<&str>) -> String {
    raw.unwrap_or_default().trim().to_string()
}

fn normalize_route_account(raw: Option<&str>) -> String {
    let trimmed = raw.unwrap_or_default().trim();
    if trimmed.is_empty() {
        return "default".to_string();
    }

    let mut out = String::new();
    let mut last_was_dash = false;
    for ch in trimmed.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
            last_was_dash = ch == '-';
        } else if !last_was_dash {
            out.push('-');
            last_was_dash = true;
        }
        if out.len() >= 64 {
            break;
        }
    }

    let normalized = out.trim_matches('-').to_string();
    if normalized.is_empty()
        || normalized == "__proto__"
        || normalized == "constructor"
        || normalized == "prototype"
    {
        "default".to_string()
    } else {
        normalized
    }
}

fn route_account_matches(pattern: &str, actual: &str) -> bool {
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return actual == "default";
    }
    if trimmed == "*" {
        return true;
    }
    normalize_route_account(Some(trimmed)) == actual
}

fn route_account_is_wildcard(account_id: &str) -> bool {
    account_id.trim() == "*"
}

fn normalize_route_peer_kind(raw: Option<&str>) -> Option<&'static str> {
    match raw.unwrap_or_default().trim().to_lowercase().as_str() {
        "direct" | "dm" => Some("direct"),
        "group" => Some("group"),
        "channel" => Some("channel"),
        _ => None,
    }
}

fn route_peer_kind_matches(binding_kind: &str, peer_kind: &str) -> bool {
    binding_kind == peer_kind
        || ((binding_kind == "group" || binding_kind == "channel")
            && (peer_kind == "group" || peer_kind == "channel"))
}

fn route_peer_matches(binding_peer: &serde_json::Value, peer: &serde_json::Value) -> bool {
    let binding_peer_id =
        normalize_route_text(binding_peer.get("id").and_then(serde_json::Value::as_str));
    if binding_peer_id.is_empty() {
        return false;
    }

    let peer_id = normalize_route_text(peer.get("id").and_then(serde_json::Value::as_str));
    if binding_peer_id != peer_id {
        return false;
    }

    let Some(binding_peer_kind) =
        normalize_route_peer_kind(binding_peer.get("kind").and_then(serde_json::Value::as_str))
    else {
        return false;
    };
    let Some(peer_kind) =
        normalize_route_peer_kind(peer.get("kind").and_then(serde_json::Value::as_str))
    else {
        return false;
    };

    route_peer_kind_matches(binding_peer_kind, peer_kind)
}

fn route_binding_has_peer_constraint(match_obj: &serde_json::Value) -> bool {
    match_obj.get("peer").is_some()
}

fn route_match_text(match_obj: &serde_json::Value, key: &str) -> String {
    normalize_route_text(match_obj.get(key).and_then(serde_json::Value::as_str))
}

fn route_match_roles(match_obj: &serde_json::Value) -> Vec<String> {
    match_obj
        .get("roles")
        .and_then(serde_json::Value::as_array)
        .map(|roles| {
            roles
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|role| !role.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn route_payload_roles(payload: &serde_json::Value) -> Vec<String> {
    payload
        .get("memberRoleIds")
        .or_else(|| payload.get("member_role_ids"))
        .or_else(|| payload.get("roles"))
        .and_then(serde_json::Value::as_array)
        .map(|roles| {
            roles
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|role| !role.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn route_match_has_guild_constraint(match_obj: &serde_json::Value) -> bool {
    !route_match_text(match_obj, "guildId").is_empty()
}

fn route_match_has_team_constraint(match_obj: &serde_json::Value) -> bool {
    !route_match_text(match_obj, "teamId").is_empty()
}

fn route_match_has_roles_constraint(match_obj: &serde_json::Value) -> bool {
    !route_match_roles(match_obj).is_empty()
}

fn route_match_has_non_peer_scope_constraint(match_obj: &serde_json::Value) -> bool {
    route_match_has_guild_constraint(match_obj)
        || route_match_has_team_constraint(match_obj)
        || route_match_has_roles_constraint(match_obj)
}

fn route_match_scope_matches(
    match_obj: &serde_json::Value,
    guild_id: &str,
    team_id: &str,
    member_role_ids: &[String],
) -> bool {
    let binding_guild_id = route_match_text(match_obj, "guildId");
    if !binding_guild_id.is_empty() && binding_guild_id != guild_id {
        return false;
    }

    let binding_team_id = route_match_text(match_obj, "teamId");
    if !binding_team_id.is_empty() && binding_team_id != team_id {
        return false;
    }

    let binding_roles = route_match_roles(match_obj);
    if binding_roles.is_empty() {
        return true;
    }

    binding_roles
        .iter()
        .any(|role| member_role_ids.iter().any(|actual| actual == role))
}

fn route_binding_matches_channel_account(
    match_obj: &serde_json::Value,
    channel: &str,
    account_id: &str,
) -> bool {
    let binding_channel =
        normalize_route_channel(match_obj.get("channel").and_then(serde_json::Value::as_str));
    let binding_account = normalize_route_text(
        match_obj
            .get("accountId")
            .or_else(|| match_obj.get("account_id"))
            .and_then(serde_json::Value::as_str),
    );

    binding_channel == channel && route_account_matches(&binding_account, account_id)
}

fn route_binding_agent_id(binding: &serde_json::Value, default_agent_id: &str) -> String {
    let agent_id = binding
        .get("agentId")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(default_agent_id)
        .trim();
    if agent_id.is_empty() {
        default_agent_id.to_string()
    } else {
        agent_id.to_string()
    }
}

fn route_choose(
    binding: &serde_json::Value,
    default_agent_id: &str,
    matched_by: &str,
) -> serde_json::Value {
    serde_json::json!({
        "agentId": route_binding_agent_id(binding, default_agent_id),
        "matchedBy": matched_by,
    })
}

pub(super) fn resolve_im_route_from_payload(payload: &serde_json::Value) -> serde_json::Value {
    let channel =
        normalize_route_channel(payload.get("channel").and_then(serde_json::Value::as_str));
    let account_id = normalize_route_account(
        payload
            .get("account_id")
            .or_else(|| payload.get("accountId"))
            .and_then(serde_json::Value::as_str),
    );
    let peer = payload
        .get("peer")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({ "kind": "group", "id": "" }));
    let parent_peer = payload
        .get("parentPeer")
        .or_else(|| payload.get("parent_peer"));
    let guild_id = normalize_route_text(payload.get("guildId").and_then(serde_json::Value::as_str));
    let team_id = normalize_route_text(payload.get("teamId").and_then(serde_json::Value::as_str));
    let member_role_ids = route_payload_roles(payload);
    let default_agent_id = payload
        .get("default_agent_id")
        .or_else(|| payload.get("defaultAgentId"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("main");
    let bindings = payload
        .get("bindings")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    for binding in &bindings {
        let Some(match_obj) = binding.get("match") else {
            continue;
        };
        let binding_peer = match_obj.get("peer").unwrap_or(&serde_json::Value::Null);
        if route_binding_matches_channel_account(match_obj, &channel, &account_id)
            && route_peer_matches(binding_peer, &peer)
            && route_match_scope_matches(match_obj, &guild_id, &team_id, &member_role_ids)
        {
            return route_choose(binding, default_agent_id, "binding.peer");
        }
    }

    if let Some(parent_peer) = parent_peer {
        for binding in &bindings {
            let Some(match_obj) = binding.get("match") else {
                continue;
            };
            let binding_peer = match_obj.get("peer").unwrap_or(&serde_json::Value::Null);
            if route_binding_matches_channel_account(match_obj, &channel, &account_id)
                && route_peer_matches(binding_peer, parent_peer)
                && route_match_scope_matches(match_obj, &guild_id, &team_id, &member_role_ids)
            {
                return route_choose(binding, default_agent_id, "binding.peer.parent");
            }
        }
    }

    for binding in &bindings {
        let Some(match_obj) = binding.get("match") else {
            continue;
        };
        if route_binding_matches_channel_account(match_obj, &channel, &account_id)
            && !route_binding_has_peer_constraint(match_obj)
            && route_match_has_guild_constraint(match_obj)
            && route_match_has_roles_constraint(match_obj)
            && route_match_scope_matches(match_obj, &guild_id, &team_id, &member_role_ids)
        {
            return route_choose(binding, default_agent_id, "binding.guild+roles");
        }
    }

    for binding in &bindings {
        let Some(match_obj) = binding.get("match") else {
            continue;
        };
        if route_binding_matches_channel_account(match_obj, &channel, &account_id)
            && !route_binding_has_peer_constraint(match_obj)
            && route_match_has_guild_constraint(match_obj)
            && !route_match_has_roles_constraint(match_obj)
            && route_match_scope_matches(match_obj, &guild_id, &team_id, &member_role_ids)
        {
            return route_choose(binding, default_agent_id, "binding.guild");
        }
    }

    for binding in &bindings {
        let Some(match_obj) = binding.get("match") else {
            continue;
        };
        if route_binding_matches_channel_account(match_obj, &channel, &account_id)
            && !route_binding_has_peer_constraint(match_obj)
            && !route_match_has_guild_constraint(match_obj)
            && route_match_has_team_constraint(match_obj)
            && route_match_scope_matches(match_obj, &guild_id, &team_id, &member_role_ids)
        {
            return route_choose(binding, default_agent_id, "binding.team");
        }
    }

    for binding in &bindings {
        let Some(match_obj) = binding.get("match") else {
            continue;
        };
        let binding_account = normalize_route_text(
            match_obj
                .get("accountId")
                .or_else(|| match_obj.get("account_id"))
                .and_then(serde_json::Value::as_str),
        );

        if route_binding_matches_channel_account(match_obj, &channel, &account_id)
            && !route_binding_has_peer_constraint(match_obj)
            && !route_match_has_non_peer_scope_constraint(match_obj)
            && !route_account_is_wildcard(&binding_account)
        {
            return route_choose(binding, default_agent_id, "binding.account");
        }
    }

    for binding in &bindings {
        let Some(match_obj) = binding.get("match") else {
            continue;
        };
        let binding_account = normalize_route_text(
            match_obj
                .get("accountId")
                .or_else(|| match_obj.get("account_id"))
                .and_then(serde_json::Value::as_str),
        );

        if route_binding_matches_channel_account(match_obj, &channel, &account_id)
            && !route_binding_has_peer_constraint(match_obj)
            && !route_match_has_non_peer_scope_constraint(match_obj)
            && route_account_is_wildcard(&binding_account)
        {
            return route_choose(binding, default_agent_id, "binding.channel");
        }
    }

    serde_json::json!({
        "agentId": default_agent_id,
        "matchedBy": "default",
    })
}
