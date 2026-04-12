import type { ComponentProps } from "react";

import { ChatView } from "./ChatView";
import { NewSessionLanding } from "./NewSessionLanding";
import { TaskTabStrip, type TaskTabStripItem } from "./TaskTabStrip";
import { ExpertCreateView } from "./experts/ExpertCreateView";
import { ExpertsView } from "./experts/ExpertsView";
import { EmployeeHubScene } from "../scenes/employees/EmployeeHubScene";
import type {
  AgentEmployee,
  ModelConfig,
  PendingAttachment,
  PersistedChatRuntimeState,
  SessionInfo,
  SkillManifest,
} from "../types";
import type { EmployeeHubOpenRequest } from "../scenes/employees/EmployeeHubScene";
import type { EmployeeAssistantSessionContext } from "../scenes/employees/employeeAssistantService";

type ChatViewProps = ComponentProps<typeof ChatView>;
type NewSessionLandingProps = ComponentProps<typeof NewSessionLanding>;
type ExpertCreateViewProps = ComponentProps<typeof ExpertCreateView>;
type ExpertsViewProps = ComponentProps<typeof ExpertsView>;
type EmployeeHubSceneProps = ComponentProps<typeof EmployeeHubScene>;

export type MainView = "start-task" | "experts" | "experts-new" | "packaging" | "employees";
export type SettingsTab =
  | "models"
  | "desktop"
  | "capabilities"
  | "health"
  | "mcp"
  | "search"
  | "routing"
  | "feishu";

export interface AppMainContentProps {
  showSettings: boolean;
  activeMainView: MainView;
  taskTabs: TaskTabStripItem[];
  activeTabId: string;
  onSelectTab: (tabId: string) => void;
  onCreateTab: () => void;
  onCloseTab: (tabId: string) => void;
  settingsInitialTab: SettingsTab;
  onCloseSettings: () => Promise<void>;
  onOpenEmployeesFromSettings: () => void;
  onDevResetFirstUseOnboarding: () => void;
  onDevOpenQuickModelSetup: () => void;
  creatingExpertSkill: boolean;
  expertCreateError: string | null;
  expertSavedPath: string | null;
  pendingImportDir: string | null;
  retryingExpertImport: boolean;
  onBackToExperts: () => void;
  onOpenPackagingView: () => void;
  onPickSkillDirectory: () => Promise<string | null>;
  onCreateExpertSkill: ExpertCreateViewProps["onSave"];
  onRetryExpertImport: () => Promise<void>;
  onRenderExpertPreview: ExpertCreateViewProps["onRenderPreview"];
  skills: SkillManifest[];
  createSessionError: string | null;
  onOpenInstallDialog: () => void;
  onOpenCreateExpertView: () => void;
  onInstallFromLibrary: ExpertsViewProps["onInstallFromLibrary"];
  onStartTaskWithSkill: (skillId: string) => Promise<void>;
  onRefreshLocalSkill: (skillId: string) => Promise<void>;
  onCheckClawhubUpdate: (skillId: string) => Promise<void>;
  onUpdateClawhubSkill: (skillId: string) => Promise<void>;
  onDeleteSkill: (skillId: string) => Promise<void>;
  clawhubUpdateStatus: Record<string, { hasUpdate: boolean; message: string }>;
  busySkillId: string | undefined;
  busyAction: "refresh" | "delete" | "check-update" | "update" | null;
  employees: AgentEmployee[];
  employeeHubOpenRequest: EmployeeHubOpenRequest | null;
  onRefreshEmployees: NonNullable<EmployeeHubSceneProps["onRefreshEmployees"]>;
  onRefreshEmployeeGroups: NonNullable<EmployeeHubSceneProps["onRefreshEmployeeGroups"]>;
  onEnterStartTask: () => void;
  onStartTaskWithEmployee: (employeeId: string) => Promise<void>;
  onOpenGroupRunSession: (sessionId: string, skillId: string) => void | Promise<void>;
  onLaunchEmployeeCreatorSkill: (options?: {
    employeeId?: string;
    employeeName?: string;
  }) => Promise<void>;
  onOpenEmployeeHubFeishuSettings: () => void;
  selectedSkill: SkillManifest | null;
  models: ModelConfig[];
  selectedSessionId: string | null;
  selectedSession: SessionInfo | null | undefined;
  selectedSessionEmployeeName?: string;
  operationPermissionMode: NonNullable<ChatViewProps["operationPermissionMode"]>;
  onOpenSession: NonNullable<ChatViewProps["onOpenSession"]>;
  sessionFocusRequest?: ChatViewProps["sessionFocusRequest"];
  groupRunStepFocusRequest?: ChatViewProps["groupRunStepFocusRequest"];
  sessionExecutionContext?: ChatViewProps["sessionExecutionContext"];
  onReturnToSourceSession: (sourceSessionId: string) => void;
  onSessionUpdate: NonNullable<ChatViewProps["onSessionUpdate"]>;
  onSessionBlockingStateChange: NonNullable<ChatViewProps["onSessionBlockingStateChange"]>;
  persistedRuntimeState?: PersistedChatRuntimeState;
  onPersistRuntimeState: (state: PersistedChatRuntimeState) => void;
  installedSkillIds: string[];
  onSkillInstalled: () => Promise<void>;
  suppressAskUserPrompt: boolean;
  initialMessage?: string;
  initialAttachments?: PendingAttachment[];
  quickPrompts: { label: string; prompt: string }[];
  employeeAssistantContext?: EmployeeAssistantSessionContext;
  onInitialMessageConsumed: () => void;
  onInitialAttachmentsConsumed: () => void;
  visibleSessions: SessionInfo[];
  landingTeams: NonNullable<NewSessionLandingProps["teams"]>;
  defaultWorkDir?: string | null;
  onCreateSessionWithInitialMessage: NewSessionLandingProps["onCreateSessionWithInitialMessage"];
  onCreateTeamEntrySession: NonNullable<NewSessionLandingProps["onCreateTeamEntrySession"]>;
  onPickLandingWorkDir: (currentWorkDir?: string) => Promise<string | null>;
  onSelectSession: (sessionId: string) => void;
  creatingSession: boolean;
}
