import { invoke } from "@tauri-apps/api/core";

import type { Message, SendMessageRequest, SessionInfo, SessionRunProjection } from "../../types";

export async function listSessions(): Promise<SessionInfo[]> {
  const sessions = await invoke<SessionInfo[] | null>("list_sessions");
  return Array.isArray(sessions) ? sessions : [];
}

export async function getMessages(sessionId: string): Promise<Message[]> {
  const messages = await invoke<Message[]>("get_messages", { sessionId });
  return Array.isArray(messages) ? messages : [];
}

export async function listSessionRuns(sessionId: string): Promise<SessionRunProjection[]> {
  const runs = await invoke<SessionRunProjection[] | null>("list_session_runs", { sessionId });
  return Array.isArray(runs) ? runs : [];
}

export async function updateSessionWorkspace(sessionId: string, workspace: string): Promise<void> {
  await invoke("update_session_workspace", {
    sessionId,
    workspace,
  });
}

export async function sendMessage(request: SendMessageRequest): Promise<void> {
  await invoke("send_message", { request });
}

export async function cancelAgent(sessionId?: string): Promise<void> {
  await invoke("cancel_agent", sessionId?.trim() ? { sessionId } : {});
}

export async function answerUserQuestion(answer: string): Promise<void> {
  await invoke("answer_user_question", { answer });
}
