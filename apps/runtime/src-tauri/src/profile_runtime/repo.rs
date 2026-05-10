use super::types::ProfileAliasCandidate;

pub(crate) async fn load_profile_alias_candidates_with_pool(
    pool: &sqlx::SqlitePool,
) -> Result<Vec<ProfileAliasCandidate>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String, String)>(
        "SELECT
            e.id,
            COALESCE(p.id, e.id),
            COALESCE(NULLIF(p.legacy_employee_row_id, ''), e.id),
            COALESCE(e.employee_id, ''),
            COALESCE(e.role_id, ''),
            COALESCE(e.openclaw_agent_id, ''),
            COALESCE(NULLIF(TRIM(p.display_name), ''), COALESCE(e.name, '')),
            COALESCE(p.profile_home, '')
         FROM agent_employees e
         LEFT JOIN agent_profiles p
           ON p.legacy_employee_row_id = e.id OR p.id = e.id",
    )
    .fetch_all(pool)
    .await;

    let rows = match rows {
        Ok(rows) => rows,
        Err(error) if is_missing_profile_runtime_table(&error) => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };

    let mut candidates = Vec::<(String, ProfileAliasCandidate, bool)>::new();
    for (
        employee_row_id,
        profile_id,
        legacy_employee_row_id,
        employee_id,
        role_id,
        openclaw_agent_id,
        display_name,
        profile_home,
    ) in rows
    {
        let candidate = ProfileAliasCandidate {
            profile_id,
            legacy_employee_row_id,
            employee_id,
            role_id,
            openclaw_agent_id,
            display_name,
        };
        let has_profile_home = !profile_home.trim().is_empty();
        if let Some(existing) = candidates
            .iter_mut()
            .find(|(row_id, _, _)| row_id == &employee_row_id)
        {
            if has_profile_home && !existing.2 {
                *existing = (employee_row_id, candidate, has_profile_home);
            }
        } else {
            candidates.push((employee_row_id, candidate, has_profile_home));
        }
    }

    Ok(candidates
        .into_iter()
        .map(|(_, candidate, _)| candidate)
        .collect())
}

fn is_missing_profile_runtime_table(error: &sqlx::Error) -> bool {
    let message = error.to_string();
    message.contains("no such table: agent_profiles")
        || message.contains("no such table: agent_employees")
}
