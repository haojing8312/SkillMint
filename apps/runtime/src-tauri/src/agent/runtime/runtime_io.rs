#![allow(unused_imports)]

mod profile_session_index;
mod runtime_events;
mod runtime_inputs;
mod runtime_support;
mod session_titles;
mod skill_os_index;
pub(crate) mod skill_source_policy;
mod types;
mod workspace_skills;

pub use profile_session_index::{
    ProfileSessionSearchFilters, ProfileSessionSearchResult,
    ensure_profile_session_index_schema_with_pool, index_profile_session_manifest_with_pool,
    refresh_profile_session_index_for_session_with_pool,
    search_profile_session_index_with_filters_with_pool, search_profile_session_index_with_pool,
};
pub(crate) use runtime_events::{
    append_partial_assistant_chunk_with_pool, append_run_failed_with_pool,
    append_run_guard_warning_with_pool, append_run_started_with_pool, append_run_stopped_with_pool,
    append_skill_route_recorded_with_pool, finalize_run_success_with_pool,
    insert_session_message_with_pool, persist_partial_assistant_message_for_run_with_pool,
    record_route_attempt_log_with_pool,
};
pub(crate) use runtime_inputs::{
    load_default_search_provider_config_with_pool, load_installed_skill_source_with_pool,
    load_session_history_with_pool, load_session_runtime_inputs_with_pool,
};
pub use runtime_support::{
    ProfileMemoryBundle, ProfileMemoryLocator, ProfileMemoryStatus,
    ProfileSessionManifestInput, build_profile_memory_locator, collect_profile_memory_status,
    load_profile_memory_bundle, load_profile_memory_bundle_with_budget,
    write_profile_session_manifest,
};
pub(crate) use runtime_support::{
    resolve_tool_name_list, resolve_tool_names,
};
pub(crate) use session_titles::{
    derive_meaningful_session_title_from_messages, is_generic_session_title,
    maybe_update_session_title_from_first_user_message_with_pool,
};
pub use skill_os_index::{
    SkillOsCapabilities, SkillOsIndexEntry, SkillOsSourceProjection, SkillOsToolsetPolicy,
    SkillOsUsageTelemetry, SkillOsVersionEntry, SkillOsVersionView, SkillOsView,
    archive_skill_os_entry_with_pool, create_agent_skill_os_entry_with_pool,
    delete_skill_os_entry_with_pool, ensure_skill_os_lifecycle_schema_with_pool,
    ensure_skill_os_versions_schema_with_pool, list_skill_os_index_with_pool,
    list_skill_os_versions_with_pool, mark_skill_os_stale_with_pool,
    patch_skill_os_entry_with_pool, record_skill_os_usage_with_pool,
    reset_skill_os_entry_with_pool, restore_skill_os_entry_with_pool,
    restore_stale_skill_os_with_pool, rollback_skill_os_entry_with_pool,
    set_skill_os_pinned_with_pool, view_skill_os_entry_with_pool, view_skill_os_version_with_pool,
};
pub use types::{
    WorkspaceSkillCommandSpec, WorkspaceSkillContent, WorkspaceSkillRouteExecutionMode,
    WorkspaceSkillRouteProjection, WorkspaceSkillRuntimeEntry,
};
pub(crate) use workspace_skills::{
    build_skill_roots, extract_assistant_text_content, extract_skill_prompt_from_decrypted_files,
    load_skill_prompt, normalize_workspace_skill_dir_name, resolve_directory_backed_skill_root,
    resolve_workspace_skill_runtime_entry, sync_workspace_skills_to_directory,
};
pub use workspace_skills::{
    build_workspace_skill_command_specs, load_workspace_skill_runtime_entries_with_pool,
};
