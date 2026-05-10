use super::types::{ProfileAliasCandidate, ProfileAliasResolution};

fn normalize_alias(raw: &str) -> String {
    raw.trim().to_lowercase()
}

pub(crate) fn resolve_profile_for_alias(
    candidates: &[ProfileAliasCandidate],
    alias: &str,
) -> Option<ProfileAliasResolution> {
    let normalized = normalize_alias(alias);
    if normalized.is_empty() {
        return None;
    }

    for candidate in candidates {
        let aliases = [
            candidate.profile_id.as_str(),
            candidate.legacy_employee_row_id.as_str(),
            candidate.employee_id.as_str(),
            candidate.role_id.as_str(),
            candidate.openclaw_agent_id.as_str(),
        ];
        if aliases
            .iter()
            .map(|value| normalize_alias(value))
            .any(|value| value == normalized)
        {
            return Some(ProfileAliasResolution {
                profile_id: candidate.profile_id.clone(),
                matched_alias: alias.trim().to_string(),
                display_name: candidate.display_name.clone(),
            });
        }
    }

    None
}

pub(crate) async fn resolve_profile_for_alias_with_pool(
    pool: &sqlx::SqlitePool,
    alias: &str,
) -> Result<Option<ProfileAliasResolution>, String> {
    let candidates =
        crate::profile_runtime::repo::load_profile_alias_candidates_with_pool(pool).await?;
    Ok(resolve_profile_for_alias(&candidates, alias))
}

#[cfg(test)]
mod tests {
    use super::{resolve_profile_for_alias, ProfileAliasCandidate};

    fn candidate() -> ProfileAliasCandidate {
        ProfileAliasCandidate {
            profile_id: "profile-1".to_string(),
            legacy_employee_row_id: "employee-row-1".to_string(),
            employee_id: "planner".to_string(),
            role_id: "planner-role".to_string(),
            openclaw_agent_id: "oc-planner".to_string(),
            display_name: "Planner".to_string(),
        }
    }

    #[test]
    fn resolves_profile_from_employee_code_role_openclaw_or_row_id() {
        for alias in [
            "planner",
            "planner-role",
            "oc-planner",
            "employee-row-1",
            "profile-1",
        ] {
            let resolved =
                resolve_profile_for_alias(&[candidate()], alias).expect("alias should resolve");
            assert_eq!(resolved.profile_id, "profile-1");
            assert_eq!(resolved.display_name, "Planner");
        }
    }

    #[test]
    fn ignores_empty_or_unknown_aliases() {
        assert!(resolve_profile_for_alias(&[candidate()], "").is_none());
        assert!(resolve_profile_for_alias(&[candidate()], "missing").is_none());
    }
}
