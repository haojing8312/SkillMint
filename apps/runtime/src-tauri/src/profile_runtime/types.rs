#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileAliasCandidate {
    pub profile_id: String,
    pub legacy_employee_row_id: String,
    pub employee_id: String,
    pub role_id: String,
    pub openclaw_agent_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileAliasResolution {
    pub profile_id: String,
    pub matched_alias: String,
    pub display_name: String,
}
