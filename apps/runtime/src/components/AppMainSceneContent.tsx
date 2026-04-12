import type { ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";

import { ChatView } from "./ChatView";
import { NewSessionLanding } from "./NewSessionLanding";
import { PackagingView } from "./packaging/PackagingView";
import { SettingsView } from "./SettingsView";
import { EmployeeHubScene } from "../scenes/employees/EmployeeHubScene";
import { ExpertCreateView } from "./experts/ExpertCreateView";
import { ExpertsView } from "./experts/ExpertsView";
import { SHOW_DEV_MODEL_SETUP_TOOLS } from "../app-shell-constants";
import type { AppMainContentProps } from "./AppMainContent.types";

function AnimatedScene({ sceneKey, children }: { sceneKey: string; children: ReactNode }) {
  return (
    <motion.div
      key={sceneKey}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="h-full"
    >
      {children}
    </motion.div>
  );
}

export function AppMainSceneContent(props: AppMainContentProps) {
  const {
    showSettings,
    activeMainView,
    settingsInitialTab,
    onCloseSettings,
    onOpenEmployeesFromSettings,
    onDevResetFirstUseOnboarding,
    onDevOpenQuickModelSetup,
    creatingExpertSkill,
    expertCreateError,
    expertSavedPath,
    pendingImportDir,
    retryingExpertImport,
    onBackToExperts,
    onOpenPackagingView,
    onPickSkillDirectory,
    onCreateExpertSkill,
    onRetryExpertImport,
    onRenderExpertPreview,
    skills,
    createSessionError,
    onOpenInstallDialog,
    onOpenCreateExpertView,
    onInstallFromLibrary,
    onStartTaskWithSkill,
    onRefreshLocalSkill,
    onCheckClawhubUpdate,
    onUpdateClawhubSkill,
    onDeleteSkill,
    clawhubUpdateStatus,
    busySkillId,
    busyAction,
    employees,
    employeeHubOpenRequest,
    onRefreshEmployees,
    onRefreshEmployeeGroups,
    onEnterStartTask,
    onStartTaskWithEmployee,
    onOpenGroupRunSession,
    onLaunchEmployeeCreatorSkill,
    onOpenEmployeeHubFeishuSettings,
    selectedSkill,
    models,
    selectedSessionId,
    selectedSession,
    selectedSessionEmployeeName,
    operationPermissionMode,
    onOpenSession,
    sessionFocusRequest,
    groupRunStepFocusRequest,
    sessionExecutionContext,
    onReturnToSourceSession,
    onSessionUpdate,
    onSessionBlockingStateChange,
    persistedRuntimeState,
    onPersistRuntimeState,
    installedSkillIds,
    onSkillInstalled,
    suppressAskUserPrompt,
    initialMessage,
    initialAttachments,
    quickPrompts,
    employeeAssistantContext,
    onInitialMessageConsumed,
    onInitialAttachmentsConsumed,
    visibleSessions,
    landingTeams,
    defaultWorkDir,
    onCreateSessionWithInitialMessage,
    onCreateTeamEntrySession,
    onPickLandingWorkDir,
    onSelectSession,
    creatingSession,
  } = props;

  return (
    <AnimatePresence mode="wait">
      {showSettings ? (
        <AnimatedScene sceneKey="settings">
          <SettingsView
            initialTab={settingsInitialTab}
            onClose={onCloseSettings}
            onOpenEmployees={onOpenEmployeesFromSettings}
            showDevModelSetupTools={SHOW_DEV_MODEL_SETUP_TOOLS}
            onDevResetFirstUseOnboarding={onDevResetFirstUseOnboarding}
            onDevOpenQuickModelSetup={onDevOpenQuickModelSetup}
          />
        </AnimatedScene>
      ) : activeMainView === "packaging" ? (
        <AnimatedScene sceneKey="packaging">
          <PackagingView />
        </AnimatedScene>
      ) : activeMainView === "experts-new" ? (
        <AnimatedScene sceneKey="experts-new">
          <ExpertCreateView
            saving={creatingExpertSkill}
            error={expertCreateError}
            savedPath={expertSavedPath}
            canRetryImport={Boolean(pendingImportDir)}
            retryingImport={retryingExpertImport}
            onBack={onBackToExperts}
            onOpenPackaging={onOpenPackagingView}
            onPickDirectory={onPickSkillDirectory}
            onSave={onCreateExpertSkill}
            onRetryImport={onRetryExpertImport}
            onRenderPreview={onRenderExpertPreview}
          />
        </AnimatedScene>
      ) : activeMainView === "experts" ? (
        <AnimatedScene sceneKey="experts">
          <ExpertsView
            skills={skills}
            launchError={createSessionError}
            onInstallSkill={onOpenInstallDialog}
            onCreate={onOpenCreateExpertView}
            onOpenPackaging={onOpenPackagingView}
            onInstallFromLibrary={onInstallFromLibrary}
            onStartTaskWithSkill={onStartTaskWithSkill}
            onRefreshLocalSkill={onRefreshLocalSkill}
            onCheckClawhubUpdate={onCheckClawhubUpdate}
            onUpdateClawhubSkill={onUpdateClawhubSkill}
            onDeleteSkill={onDeleteSkill}
            clawhubUpdateStatus={clawhubUpdateStatus}
            busySkillId={busySkillId}
            busyAction={busyAction}
          />
        </AnimatedScene>
      ) : activeMainView === "employees" ? (
        <AnimatedScene sceneKey="employees">
          <EmployeeHubScene
            employees={employees}
            skills={skills}
            openRequest={employeeHubOpenRequest}
            onRefreshEmployees={onRefreshEmployees}
            onRefreshEmployeeGroups={onRefreshEmployeeGroups}
            onEnterStartTask={onEnterStartTask}
            onStartTaskWithEmployee={onStartTaskWithEmployee}
            onOpenGroupRunSession={onOpenGroupRunSession}
            onLaunchEmployeeCreatorSkill={onLaunchEmployeeCreatorSkill}
            onOpenFeishuSettingsPanel={onOpenEmployeeHubFeishuSettings}
          />
        </AnimatedScene>
      ) : selectedSkill && models.length > 0 && selectedSessionId ? (
        <AnimatedScene sceneKey="chat">
          <ChatView
            skill={selectedSkill}
            models={models}
            sessionId={selectedSessionId}
            sessionModelId={selectedSession?.model_id}
            workDir={selectedSession?.work_dir}
            onOpenSession={onOpenSession}
            sessionFocusRequest={sessionFocusRequest}
            groupRunStepFocusRequest={groupRunStepFocusRequest}
            sessionExecutionContext={sessionExecutionContext}
            onReturnToSourceSession={onReturnToSourceSession}
            sessionSourceChannel={selectedSession?.source_channel}
            sessionSourceLabel={selectedSession?.source_label}
            sessionTitle={selectedSession?.display_title || selectedSession?.title}
            sessionMode={selectedSession?.session_mode}
            sessionEmployeeName={selectedSessionEmployeeName}
            operationPermissionMode={operationPermissionMode}
            onSessionUpdate={onSessionUpdate}
            onSessionBlockingStateChange={onSessionBlockingStateChange}
            persistedRuntimeState={persistedRuntimeState}
            onPersistRuntimeState={onPersistRuntimeState}
            installedSkillIds={installedSkillIds}
            onSkillInstalled={onSkillInstalled}
            suppressAskUserPrompt={suppressAskUserPrompt}
            initialMessage={initialMessage}
            initialAttachments={initialAttachments}
            quickPrompts={quickPrompts}
            employeeAssistantContext={employeeAssistantContext}
            onInitialMessageConsumed={onInitialMessageConsumed}
            onInitialAttachmentsConsumed={onInitialAttachmentsConsumed}
          />
        </AnimatedScene>
      ) : selectedSkill && models.length > 0 ? (
        <AnimatedScene sceneKey="new-session">
          <NewSessionLanding
            sessions={visibleSessions}
            teams={landingTeams}
            creating={creatingSession}
            error={createSessionError}
            defaultWorkDir={defaultWorkDir ?? undefined}
            onSelectSession={onSelectSession}
            onCreateSessionWithInitialMessage={onCreateSessionWithInitialMessage}
            onCreateTeamEntrySession={onCreateTeamEntrySession}
            onPickWorkDir={onPickLandingWorkDir}
          />
        </AnimatedScene>
      ) : selectedSkill && models.length === 0 ? (
        <div className="flex items-center justify-center h-full sm-text-muted text-sm">
          请先在设置中配置模型和 API Key
        </div>
      ) : (
        <div className="flex items-center justify-center h-full sm-text-muted text-sm">
          从左侧选择一个技能，开始任务
        </div>
      )}
    </AnimatePresence>
  );
}
