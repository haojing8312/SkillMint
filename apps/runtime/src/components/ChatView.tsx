import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import { SkillManifest, ModelConfig, Message } from "../types";

interface Props {
  skill: SkillManifest;
  models: ModelConfig[];
}

export function ChatView({ skill, models }: Props) {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [streaming, setStreaming] = useState(false);
  const [streamBuffer, setStreamBuffer] = useState("");
  const [selectedModelId, setSelectedModelId] = useState(models[0]?.id ?? "");
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    startNewSession();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [skill.id]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamBuffer]);

  useEffect(() => {
    let currentSessionId: string | null = sessionId;
    const unlistenPromise = listen<{ session_id: string; token: string; done: boolean }>(
      "stream-token",
      ({ payload }) => {
        if (payload.session_id !== currentSessionId) return;
        if (payload.done) {
          setMessages((prev) => [
            ...prev,
            { role: "assistant", content: payload.token, created_at: new Date().toISOString() },
          ]);
          setStreamBuffer("");
          setStreaming(false);
        } else {
          setStreamBuffer((b) => b + payload.token);
        }
      }
    );
    return () => {
      currentSessionId = null;
      unlistenPromise.then((fn) => fn());
    };
  }, [sessionId]);

  async function startNewSession() {
    const modelId = selectedModelId || models[0]?.id;
    if (!modelId) return;
    const id = await invoke<string>("create_session", {
      skillId: skill.id,
      modelId,
    });
    setSessionId(id);
    setMessages([]);
    setStreamBuffer("");
  }

  async function handleSend() {
    if (!input.trim() || streaming || !sessionId) return;
    const msg = input.trim();
    setInput("");
    setMessages((prev) => [...prev, { role: "user", content: msg, created_at: new Date().toISOString() }]);
    setStreaming(true);
    setStreamBuffer("");
    try {
      await invoke("send_message", { sessionId, userMessage: msg });
    } catch (e) {
      setStreaming(false);
      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: "错误: " + String(e), created_at: new Date().toISOString() },
      ]);
    }
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-6 py-3 border-b border-slate-700 bg-slate-800">
        <div>
          <span className="font-medium">{skill.name}</span>
          <span className="text-xs text-slate-400 ml-2">v{skill.version}</span>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={selectedModelId}
            onChange={(e) => setSelectedModelId(e.target.value)}
            className="bg-slate-700 text-sm rounded px-2 py-1 border border-slate-600 focus:outline-none"
          >
            {models.map((m) => (
              <option key={m.id} value={m.id}>{m.name}</option>
            ))}
          </select>
          <button
            onClick={startNewSession}
            className="text-sm bg-slate-700 hover:bg-slate-600 px-3 py-1 rounded transition-colors"
          >
            新建会话
          </button>
        </div>
      </div>

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
              {m.role === "assistant" ? (
                <ReactMarkdown>{m.content}</ReactMarkdown>
              ) : (
                m.content
              )}
            </div>
          </div>
        ))}
        {streamBuffer && (
          <div className="flex justify-start">
            <div className="max-w-2xl bg-slate-700 rounded-lg px-4 py-2 text-sm text-slate-100">
              <ReactMarkdown>{streamBuffer}</ReactMarkdown>
              <span className="animate-pulse">|</span>
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

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
