import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import { SkillManifest, ModelConfig, Message, ToolCallInfo } from "../types";
import { ToolCallCard } from "./ToolCallCard";

interface Props {
  skill: SkillManifest;
  models: ModelConfig[];
  sessionId: string;
  onSessionUpdate?: () => void;
}

export function ChatView({ skill, models, sessionId, onSessionUpdate }: Props) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [streaming, setStreaming] = useState(false);
  const [streamBuffer, setStreamBuffer] = useState("");
  const [currentToolCalls, setCurrentToolCalls] = useState<ToolCallInfo[]>([]);
  const bottomRef = useRef<HTMLDivElement>(null);
  const streamBufferRef = useRef("");
  const currentToolCallsRef = useRef<ToolCallInfo[]>([]);

  // sessionId 变化时加载历史消息
  useEffect(() => {
    loadMessages(sessionId);
    // 切换会话时重置流式状态
    setStreaming(false);
    setStreamBuffer("");
    streamBufferRef.current = "";
    setCurrentToolCalls([]);
    currentToolCallsRef.current = [];
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamBuffer]);

  // stream-token 事件监听（依赖 sessionId prop）
  useEffect(() => {
    let currentSessionId: string | null = sessionId;
    const unlistenPromise = listen<{ session_id: string; token: string; done: boolean }>(
      "stream-token",
      ({ payload }) => {
        if (payload.session_id !== currentSessionId) return;
        if (payload.done) {
          const finalContent = streamBufferRef.current;
          const toolCalls =
            currentToolCallsRef.current.length > 0
              ? [...currentToolCallsRef.current]
              : undefined;
          if (finalContent || toolCalls) {
            setMessages((prev) => [
              ...prev,
              {
                role: "assistant",
                content: finalContent,
                created_at: new Date().toISOString(),
                toolCalls,
              },
            ]);
          }
          currentToolCallsRef.current = [];
          setCurrentToolCalls([]);
          streamBufferRef.current = "";
          setStreamBuffer("");
          setStreaming(false);
        } else {
          streamBufferRef.current += payload.token;
          setStreamBuffer(streamBufferRef.current);
        }
      }
    );
    return () => {
      currentSessionId = null;
      unlistenPromise.then((fn) => fn());
    };
  }, [sessionId]);

  // tool-call-event 事件监听（依赖 sessionId prop）
  useEffect(() => {
    const unlistenPromise = listen<{
      session_id: string;
      tool_name: string;
      tool_input: Record<string, unknown>;
      tool_output: string | null;
      status: string;
    }>("tool-call-event", ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      if (payload.status === "started") {
        setCurrentToolCalls((prev) => {
          const next = [
            ...prev,
            {
              id: `${payload.tool_name}-${Date.now()}`,
              name: payload.tool_name,
              input: payload.tool_input,
              status: "running" as const,
            },
          ];
          currentToolCallsRef.current = next;
          return next;
        });
      } else {
        setCurrentToolCalls((prev) => {
          const next = prev.map((tc) =>
            tc.name === payload.tool_name && tc.status === "running"
              ? {
                  ...tc,
                  output: payload.tool_output ?? undefined,
                  status: (payload.status === "completed"
                    ? "completed"
                    : "error") as "completed" | "error",
                }
              : tc
          );
          currentToolCallsRef.current = next;
          return next;
        });
      }
    });
    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [sessionId]);

  async function loadMessages(sid: string) {
    try {
      const list = await invoke<Message[]>("get_messages", { sessionId: sid });
      setMessages(list);
    } catch (e) {
      console.error("加载历史消息失败:", e);
      setMessages([]);
    }
  }

  async function handleSend() {
    if (!input.trim() || streaming || !sessionId) return;
    const msg = input.trim();
    setInput("");
    setMessages((prev) => [
      ...prev,
      { role: "user", content: msg, created_at: new Date().toISOString() },
    ]);
    setStreaming(true);
    currentToolCallsRef.current = [];
    setCurrentToolCalls([]);
    streamBufferRef.current = "";
    setStreamBuffer("");
    try {
      await invoke("send_message", { sessionId, userMessage: msg });
      onSessionUpdate?.();
    } catch (e) {
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content: "错误: " + String(e),
          created_at: new Date().toISOString(),
        },
      ]);
    } finally {
      setStreaming(false);
    }
  }

  // 从 models 查找当前会话的模型名称（用于头部展示）
  const currentModel = models[0];

  return (
    <div className="flex flex-col h-full">
      {/* 头部：简化，仅显示 skill 名称和模型信息 */}
      <div className="flex items-center justify-between px-6 py-3 border-b border-slate-700 bg-slate-800">
        <div>
          <span className="font-medium">{skill.name}</span>
          <span className="text-xs text-slate-400 ml-2">v{skill.version}</span>
        </div>
        {currentModel && (
          <span className="text-xs text-slate-400">{currentModel.name}</span>
        )}
      </div>

      {/* 消息列表 */}
      <div className="flex-1 overflow-y-auto p-6 space-y-4">
        {messages.map((m, i) => (
          <div key={i} className={"flex " + (m.role === "user" ? "justify-end" : "justify-start")}>
            <div
              className={
                "max-w-2xl rounded-lg px-4 py-2 text-sm " +
                (m.role === "user"
                  ? "bg-blue-600 text-white"
                  : "bg-slate-700 text-slate-100")
              }
            >
              {m.role === "assistant" && m.toolCalls && (
                <div className="mb-2">
                  {m.toolCalls.map((tc) => (
                    <ToolCallCard key={tc.id} toolCall={tc} />
                  ))}
                </div>
              )}
              {m.role === "assistant" ? (
                <ReactMarkdown>{m.content}</ReactMarkdown>
              ) : (
                m.content
              )}
            </div>
          </div>
        ))}
        {/* 流式输出区域 */}
        {(currentToolCalls.length > 0 || streamBuffer) && (
          <div className="flex justify-start">
            <div className="max-w-2xl bg-slate-700 rounded-lg px-4 py-2 text-sm text-slate-100">
              {currentToolCalls.map((tc) => (
                <ToolCallCard key={tc.id} toolCall={tc} />
              ))}
              {streamBuffer && <ReactMarkdown>{streamBuffer}</ReactMarkdown>}
              <span className="animate-pulse">|</span>
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* 输入区域 */}
      <div className="px-6 py-4 border-t border-slate-700 bg-slate-800">
        <div className="flex gap-2">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder=""
            rows={1}
            className="flex-1 bg-slate-700 border border-slate-600 rounded px-3 py-2 text-sm resize-none focus:outline-none focus:border-blue-500"
          />
          <button
            onClick={handleSend}
            disabled={streaming || !input.trim()}
            className="bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 px-4 rounded text-sm font-medium transition-colors"
          >
            发送
          </button>
        </div>
      </div>
    </div>
  );
}
