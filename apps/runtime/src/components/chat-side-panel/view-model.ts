import type { Message, ToolCallInfo } from "../../types";
import { getToolResultErrorText, getToolResultSummary } from "../../lib/tool-result";
import {
  buildRunningToolTitle,
  classifyDeliverable,
  flattenToolCalls,
  getToolDisplayLabel,
  inferCurrentTaskTitle,
  normalizeTaskStatus,
  parseWebSearchResults,
  readTouchedPath,
} from "./view-model-helpers";

export interface TaskItemView {
  id: string;
  title: string;
  status: "pending" | "in_progress" | "completed";
  priority: string;
}

export interface TaskPanelViewModel {
  hasTodoList: boolean;
  totalTasks: number;
  completedTasks: number;
  inProgressTasks: number;
  currentTaskTitle: string;
  items: TaskItemView[];
  touchedFileCount: number;
  webSearchCount: number;
  latestTouchedFile: string;
  latestSearchQuery: string;
}

export interface SessionTouchedFile {
  path: string;
  tool: "write_file" | "edit";
}

export interface WebSearchResultView {
  title: string;
  url: string;
  snippet: string;
  domain: string;
}

export interface WebSearchEntryView {
  id: string;
  query: string;
  status: "running" | "completed" | "error";
  results: WebSearchResultView[];
  rawOutput?: string;
}

export interface TaskJourneyStepView {
  id: string;
  kind: "planning" | "research" | "delivery" | "error";
  title: string;
  detail: string;
  status: "running" | "completed" | "error";
  count: number;
}

export interface DeliverableView {
  path: string;
  tool: "write_file" | "edit";
  category: "primary" | "secondary";
}

export interface TaskJourneyViewModel {
  status: "running" | "completed" | "failed" | "partial";
  currentTaskTitle: string;
  steps: TaskJourneyStepView[];
  deliverables: DeliverableView[];
  warnings: string[];
}

export function extractSessionTouchedFiles(messages: Message[]): SessionTouchedFile[] {
  return flattenToolCalls(messages)
    .filter((tc) => tc.name === "write_file" || tc.name === "edit")
    .map((tc) => {
      const path = readTouchedPath(tc);
      return path
        ? {
            path,
            tool: tc.name as "write_file" | "edit",
          }
        : null;
    })
    .filter((item): item is SessionTouchedFile => Boolean(item));
}

export function buildTaskPanelViewModel(messages: Message[]): TaskPanelViewModel {
  const toolCalls = flattenToolCalls(messages);
  const todoCalls = toolCalls.filter((tc) => tc.name === "todo_write");
  const latestTodoCall = todoCalls[todoCalls.length - 1];
  const todos = Array.isArray(latestTodoCall?.input?.todos) ? latestTodoCall.input.todos : [];
  const todoItems: TaskItemView[] = todos.map((todo: any, index) => ({
    id: String(todo?.id || `todo-${index}`),
    title: String(todo?.content || "(无标题任务)"),
    status: normalizeTaskStatus(todo?.status),
    priority: String(todo?.priority || "medium"),
  }));
  const latestRunningTool = [...toolCalls].reverse().find((tc) => tc.status === "running");
  const items = todoItems;
  const completedTasks = items.filter((item) => item.status === "completed").length;
  const inProgressItems = items.filter((item) => item.status === "in_progress");
  const touchedFiles = extractSessionTouchedFiles(messages);
  const webSearches = buildWebSearchViewModel(messages);

  return {
    hasTodoList: todoItems.length > 0,
    totalTasks: items.length,
    completedTasks,
    inProgressTasks: inProgressItems.length,
    currentTaskTitle:
      buildRunningToolTitle(latestRunningTool) ||
      inProgressItems[0]?.title ||
      items.find((item) => item.status === "pending")?.title ||
      "",
    items,
    touchedFileCount: touchedFiles.length,
    webSearchCount: webSearches.length,
    latestTouchedFile: touchedFiles[touchedFiles.length - 1]?.path || "",
    latestSearchQuery: webSearches[webSearches.length - 1]?.query || "",
  };
}

export function buildWebSearchViewModel(messages: Message[]): WebSearchEntryView[] {
  return flattenToolCalls(messages)
    .filter((tc) => tc.name === "web_search")
    .map((tc, index) => ({
      id: tc.id || `web-search-${index}`,
      query: String(tc.input?.query || ""),
      status: tc.status,
      results: parseWebSearchResults(tc.output),
      rawOutput: tc.output,
    }))
    .filter((entry) => entry.query || entry.results.length > 0);
}

export function buildTaskJourneyViewModel(messages: Message[]): TaskJourneyViewModel {
  const toolCalls = flattenToolCalls(messages);
  const todoCalls = toolCalls.filter((tc) => tc.name === "todo_write");
  const latestTodoCall = todoCalls[todoCalls.length - 1];
  const todos = Array.isArray(latestTodoCall?.input?.todos) ? latestTodoCall.input.todos : [];
  const inProgressTodo = todos.find((todo: any) => todo?.status === "in_progress");

  const deliverables = toolCalls
    .filter((tc) => (tc.name === "write_file" || tc.name === "edit") && tc.status === "completed")
    .map((tc) => {
      const path = readTouchedPath(tc);
      return path
        ? {
            path,
            tool: tc.name as "write_file" | "edit",
            category: classifyDeliverable(path) as DeliverableView["category"],
          }
        : null;
    })
    .filter((item): item is DeliverableView => Boolean(item));

  const steps: TaskJourneyStepView[] = [];
  const warnings: string[] = [];

  for (let index = 0; index < toolCalls.length; index += 1) {
    const toolCall = toolCalls[index];
    const output = getToolResultSummary(toolCall.output).trim();

    if (toolCall.status === "error") {
      let count = 1;
      while (index + count < toolCalls.length) {
        const next = toolCalls[index + count];
        if (
          next.name !== toolCall.name ||
          next.status !== "error" ||
          getToolResultErrorText(next.output).trim() !== getToolResultErrorText(toolCall.output).trim()
        ) {
          break;
        }
        count += 1;
      }
      const errorText = getToolResultErrorText(toolCall.output).trim();
      const warning = `${toolCall.name} 失败 ${count} 次：${errorText || "未知错误"}`;
      const displayName = getToolDisplayLabel(toolCall.name);
      warnings.push(warning);
      steps.push({
        id: toolCall.id,
        kind: "error",
        title: `${displayName}失败，已重试 ${count} 次`,
        detail: warning,
        status: "error",
        count,
      });
      index += count - 1;
      continue;
    }

    if (toolCall.name === "web_search") {
      steps.push({
        id: toolCall.id,
        kind: "research",
        title: "已完成资料搜索",
        detail: String(toolCall.input?.query || ""),
        status: toolCall.status,
        count: 1,
      });
      continue;
    }

    if (toolCall.name === "todo_write") {
      steps.push({
        id: toolCall.id,
        kind: "planning",
        title: "已更新任务清单",
        detail: `${todos.length} 个任务项`,
        status: toolCall.status,
        count: 1,
      });
      continue;
    }

    if (toolCall.name === "write_file" || toolCall.name === "edit") {
      const path = readTouchedPath(toolCall);
      steps.push({
        id: toolCall.id,
        kind: "delivery",
        title: "已生成交付文件",
        detail: path,
        status: toolCall.status,
        count: 1,
      });
    }
  }

  const hasError = warnings.length > 0;
  const hasDeliverable = deliverables.length > 0;
  const hasRunning = toolCalls.some((toolCall) => toolCall.status === "running");

  return {
    status: hasRunning ? "running" : hasError ? (hasDeliverable ? "partial" : "failed") : "completed",
    currentTaskTitle:
      String(inProgressTodo?.content || "") || inferCurrentTaskTitle(toolCalls),
    steps,
    deliverables,
    warnings,
  };
}
