import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { SkillManifest, ModelConfig, Message, StreamItem } from "../types";
import { motion, AnimatePresence } from "framer-motion";
import { ToolIsland } from "./ToolIsland";

interface Props {
  skill: SkillManifest;
  models: ModelConfig[];
  sessionId: string;
  workDir?: string;
  onSessionUpdate?: () => void;
}

export function ChatView({ skill, models, sessionId, workDir, onSessionUpdate }: Props) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [streaming, setStreaming] = useState(false);
  // 有序的流式输出项：文字和工具调用按时间顺序排列
  const [streamItems, setStreamItems] = useState<StreamItem[]>([]);
  const streamItemsRef = useRef<StreamItem[]>([]);
  const [askUserQuestion, setAskUserQuestion] = useState<string | null>(null);
  const [askUserOptions, setAskUserOptions] = useState<string[]>([]);
  const [askUserAnswer, setAskUserAnswer] = useState("");
  const [agentState, setAgentState] = useState<{
    state: string;
    detail?: string;
    iteration: number;
  } | null>(null);
  const [toolConfirm, setToolConfirm] = useState<{
    toolName: string;
    toolInput: Record<string, unknown>;
  } | null>(null);
  const [subAgentBuffer, setSubAgentBuffer] = useState("");
  const bottomRef = useRef<HTMLDivElement>(null);
  const subAgentBufferRef = useRef("");

  // sessionId 变化时加载历史消息
  useEffect(() => {
    loadMessages(sessionId);
    // 切换会话时重置流式状态
    setStreaming(false);
    setStreamItems([]);
    streamItemsRef.current = [];
    setSubAgentBuffer("");
    subAgentBufferRef.current = "";
    setAskUserQuestion(null);
    setAskUserOptions([]);
    setAskUserAnswer("");
    setAgentState(null);
    setToolConfirm(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamItems, askUserQuestion, toolConfirm]);

  // stream-token 事件监听
  useEffect(() => {
    let currentSessionId: string | null = sessionId;
    const unlistenPromise = listen<{
      session_id: string;
      token: string;
      done: boolean;
      sub_agent?: boolean;
    }>(
      "stream-token",
      ({ payload }) => {
        if (payload.session_id !== currentSessionId) return;
        if (payload.done) {
          // 流结束：将 streamItems 转为历史消息
          const items = streamItemsRef.current;
          const finalText = items
            .filter((i) => i.type === "text")
            .map((i) => i.content || "")
            .join("");
          const toolCalls = items
            .filter((i) => i.type === "tool_call" && i.toolCall)
            .map((i) => i.toolCall!);
          if (finalText || toolCalls.length > 0) {
            setMessages((prev) => [
              ...prev,
              {
                role: "assistant",
                content: finalText,
                created_at: new Date().toISOString(),
                toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
                streamItems: items.length > 0 ? [...items] : undefined,
              },
            ]);
          }
          streamItemsRef.current = [];
          setStreamItems([]);
          subAgentBufferRef.current = "";
          setSubAgentBuffer("");
          setStreaming(false);
        } else if (payload.sub_agent) {
          // 子 Agent 的 token 单独缓冲
          subAgentBufferRef.current += payload.token;
          setSubAgentBuffer(subAgentBufferRef.current);
        } else {
          // 主 Agent 的文字 token → 追加到最后一个 text 项或新建
          const items = streamItemsRef.current;
          const last = items[items.length - 1];
          if (last && last.type === "text") {
            last.content = (last.content || "") + payload.token;
          } else {
            items.push({ type: "text", content: payload.token });
          }
          streamItemsRef.current = items;
          setStreamItems([...items]);
        }
      }
    );
    return () => {
      currentSessionId = null;
      unlistenPromise.then((fn) => fn());
    };
  }, [sessionId]);

  // ask-user-event 事件监听
  useEffect(() => {
    const unlistenPromise = listen<{
      session_id: string;
      question: string;
      options: string[];
    }>("ask-user-event", ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      setAskUserQuestion(payload.question);
      setAskUserOptions(payload.options);
    });
    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [sessionId]);

  // agent-state-event 事件监听
  useEffect(() => {
    const unlistenPromise = listen<{
      session_id: string;
      state: string;
      detail: string | null;
      iteration: number;
    }>("agent-state-event", ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      if (payload.state === "finished") {
        setAgentState(null);
      } else {
        setAgentState({
          state: payload.state,
          detail: payload.detail ?? undefined,
          iteration: payload.iteration,
        });
      }
    });
    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [sessionId]);

  // tool-confirm-event 事件监听（权限确认）
  useEffect(() => {
    const unlistenPromise = listen<{
      session_id: string;
      tool_name: string;
      tool_input: Record<string, unknown>;
    }>("tool-confirm-event", ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      setToolConfirm({
        toolName: payload.tool_name,
        toolInput: payload.tool_input,
      });
    });
    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [sessionId]);

  // tool-call-event 事件监听：按顺序插入到 streamItems
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
        // 新的工具调用 → 直接追加到 streamItems（文字和工具按时间排列）
        const items = streamItemsRef.current;
        items.push({
          type: "tool_call",
          toolCall: {
            id: `${payload.tool_name}-${Date.now()}`,
            name: payload.tool_name,
            input: payload.tool_input,
            status: "running" as const,
          },
        });
        streamItemsRef.current = items;
        setStreamItems([...items]);
      } else {
        // 工具完成/出错 → 更新对应项
        const items = streamItemsRef.current.map((item) => {
          if (
            item.type === "tool_call" &&
            item.toolCall?.name === payload.tool_name &&
            item.toolCall?.status === "running"
          ) {
            return {
              ...item,
              toolCall: {
                ...item.toolCall,
                output: payload.tool_output ?? undefined,
                status: (payload.status === "completed"
                  ? "completed"
                  : "error") as "completed" | "error",
              },
            };
          }
          return item;
        });
        streamItemsRef.current = items;
        setStreamItems([...items]);
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
    streamItemsRef.current = [];
    setStreamItems([]);
    subAgentBufferRef.current = "";
    setSubAgentBuffer("");
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

  async function handleCancel() {
    try {
      await invoke("cancel_agent");
    } catch (e) {
      console.error("取消任务失败:", e);
    }
    // 即时清除状态，不等待后端返回
    setStreaming(false);
    setAgentState(null);
    // 将所有 running 状态的工具标记为 error，避免永远转圈
    const items = streamItemsRef.current.map((item) => {
      if (
        item.type === "tool_call" &&
        item.toolCall?.status === "running"
      ) {
        return {
          ...item,
          toolCall: {
            ...item.toolCall,
            output: "已取消",
            status: "error" as const,
          },
        };
      }
      return item;
    });
    streamItemsRef.current = items;
    setStreamItems([...items]);
  }

  async function handleAnswerUser(answer: string) {
    if (!answer.trim()) return;
    try {
      await invoke("answer_user_question", { answer: answer.trim() });
    } catch (e) {
      console.error("回答用户问题失败:", e);
    }
    setAskUserQuestion(null);
    setAskUserOptions([]);
    setAskUserAnswer("");
  }

  async function handleToolConfirm(confirmed: boolean) {
    try {
      await invoke("confirm_tool_execution", { confirmed });
    } catch (e) {
      console.error("工具确认失败:", e);
    }
    setToolConfirm(null);
  }

  // 从 models 查找当前会话的模型名称
  const currentModel = models[0];

  // Markdown 代码块语法高亮配置
  const markdownComponents = {
    code({ className, children, ...props }: any) {
      const match = /language-(\w+)/.exec(className || "");
      const codeString = String(children).replace(/\n$/, "");
      return match ? (
        <SyntaxHighlighter
          style={oneDark}
          language={match[1]}
          PreTag="div"
          customStyle={{ margin: 0, borderRadius: "0.375rem", fontSize: "0.8125rem" }}
        >
          {codeString}
        </SyntaxHighlighter>
      ) : (
        <code className={"bg-gray-200/60 px-1.5 py-0.5 rounded text-sm text-gray-800 " + (className || "")} {...props}>
          {children}
        </code>
      );
    },
  };

  /** 渲染有序的 StreamItem 列表（将连续的工具调用合并到一个 ToolIsland） */
  function renderStreamItems(items: StreamItem[], isStreaming: boolean) {
    const groups: { type: "text" | "tools"; items: StreamItem[] }[] = [];
    for (const item of items) {
      if (item.type === "tool_call") {
        const last = groups[groups.length - 1];
        if (last && last.type === "tools") {
          last.items.push(item);
        } else {
          groups.push({ type: "tools", items: [item] });
        }
      } else {
        groups.push({ type: "text", items: [item] });
      }
    }

    return groups.map((g, i) => {
      if (g.type === "tools") {
        const toolCalls = g.items
          .filter((it) => it.toolCall)
          .map((it) => it.toolCall!);
        const hasRunning = toolCalls.some((tc) => tc.status === "running");
        return (
          <ToolIsland
            key={`island-${i}`}
            toolCalls={toolCalls}
            isRunning={hasRunning}
            subAgentBuffer={hasRunning ? subAgentBuffer : undefined}
          />
        );
      }
      const text = g.items.map((it) => it.content || "").join("");
      if (!text) return null;
      return (
        <div key={`txt-${i}`}>
          <ReactMarkdown components={markdownComponents}>{text}</ReactMarkdown>
        </div>
      );
    });
  }

  return (
    <div className="flex flex-col h-full">
      {/* 头部 */}
      <div className="flex items-center justify-between px-6 py-3.5 border-b border-gray-200 bg-white/70 backdrop-blur-sm">
        <span className="font-semibold text-gray-900">{skill.name}</span>
        <div className="flex items-center gap-3">
          {currentModel && (
            <span className="text-xs text-gray-400">{currentModel.name}</span>
          )}
        </div>
      </div>

      {/* 消息列表 */}
      <div className="flex-1 overflow-y-auto p-6 space-y-5">
        {agentState && (
          <div className="sticky top-0 z-10 flex items-center gap-2 bg-white/80 backdrop-blur-lg px-4 py-2 rounded-xl text-xs text-gray-600 border border-gray-200 shadow-sm mx-4 mt-2">
            <span className="animate-spin h-3 w-3 border-2 border-blue-400 border-t-transparent rounded-full" />
            {agentState.state === "thinking" && "思考中..."}
            {agentState.state === "tool_calling" && `执行工具: ${agentState.detail}`}
            {agentState.state === "error" && (
              <span className="text-red-400">错误: {agentState.detail}</span>
            )}
          </div>
        )}
        {messages.map((m, i) => {
          const isLatest = i === messages.length - 1;
          return (
            <motion.div
              key={i}
              initial={isLatest ? { opacity: 0, x: m.role === "user" ? 20 : -20 } : false}
              animate={{ opacity: 1, x: 0 }}
              transition={{ type: "spring", stiffness: 300, damping: 24 }}
              className={"flex " + (m.role === "user" ? "justify-end" : "justify-start")}
            >
              <div
                className={
                  "max-w-[80%] rounded-2xl px-5 py-3 text-sm " +
                  (m.role === "user"
                    ? "bg-blue-500 text-white"
                    : "bg-white text-gray-800 shadow-sm border border-gray-100")
                }
              >
                {m.role === "assistant" && m.streamItems ? (
                  renderStreamItems(m.streamItems, false)
                ) : m.role === "assistant" && m.toolCalls ? (
                  <>
                    <ToolIsland toolCalls={m.toolCalls} isRunning={false} />
                    <ReactMarkdown components={markdownComponents}>{m.content}</ReactMarkdown>
                  </>
                ) : m.role === "assistant" ? (
                  <ReactMarkdown components={markdownComponents}>{m.content}</ReactMarkdown>
                ) : (
                  m.content
                )}
              </div>
            </motion.div>
          );
        })}
        {/* 流式输出区域：按时间顺序渲染 */}
        {streamItems.length > 0 && (
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            className="flex justify-start"
          >
            <div className="max-w-[80%] bg-white rounded-2xl px-5 py-3 text-sm text-gray-800 shadow-sm border border-gray-100">
              {renderStreamItems(streamItems, true)}
              <span className="animate-pulse text-blue-400">|</span>
            </div>
          </motion.div>
        )}
        {/* AskUser 问答卡片 */}
        {askUserQuestion && (
          <div className="flex justify-start">
            <div className="max-w-[80%] bg-amber-50 border border-amber-200 rounded-2xl px-4 py-3 text-sm">
              <div className="font-medium text-amber-700 mb-2">{askUserQuestion}</div>
              {askUserOptions.length > 0 && (
                <div className="flex flex-wrap gap-2 mb-2">
                  {askUserOptions.map((opt, i) => (
                    <button
                      key={i}
                      onClick={() => handleAnswerUser(opt)}
                      className="bg-amber-100 hover:bg-amber-200 text-amber-700 px-3 py-1 rounded text-xs transition-colors"
                    >
                      {opt}
                    </button>
                  ))}
                </div>
              )}
              <div className="flex gap-2">
                <input
                  value={askUserAnswer}
                  onChange={(e) => setAskUserAnswer(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      handleAnswerUser(askUserAnswer);
                    }
                  }}
                  placeholder="输入回答..."
                  className="flex-1 bg-white border border-gray-200 rounded px-2 py-1 text-xs focus:outline-none focus:border-amber-500"
                />
                <button
                  onClick={() => handleAnswerUser(askUserAnswer)}
                  disabled={!askUserAnswer.trim()}
                  className="bg-amber-500 hover:bg-amber-600 disabled:bg-gray-200 disabled:text-gray-400 px-3 py-1 rounded text-xs transition-colors"
                >
                  回答
                </button>
              </div>
            </div>
          </div>
        )}
        {/* 工具确认卡片 */}
        {toolConfirm && (
          <div className="flex justify-start">
            <div className="max-w-[80%] bg-orange-50 border border-orange-200 rounded-2xl px-4 py-3 text-sm">
              <div className="font-medium text-orange-700 mb-2">需要确认</div>
              <div className="text-gray-600 mb-1">
                工具: <span className="text-orange-600 font-mono">{toolConfirm.toolName}</span>
              </div>
              <pre className="bg-gray-50 rounded-xl p-2.5 text-xs text-gray-600 mb-3 overflow-x-auto max-h-40 overflow-y-auto">
                {JSON.stringify(toolConfirm.toolInput, null, 2)}
              </pre>
              <div className="flex gap-2">
                <button
                  onClick={() => handleToolConfirm(true)}
                  className="bg-green-600 hover:bg-green-700 text-white px-4 py-1 rounded text-xs font-medium transition-colors"
                >
                  允许
                </button>
                <button
                  onClick={() => handleToolConfirm(false)}
                  className="bg-red-600 hover:bg-red-700 text-white px-4 py-1 rounded text-xs font-medium transition-colors"
                >
                  拒绝
                </button>
              </div>
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* 输入区域 */}
      <div className="px-6 py-4 bg-gray-50">
        <div className="relative max-w-3xl mx-auto">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder="输入消息..."
            rows={1}
            className="w-full bg-white border border-gray-200 rounded-xl pl-4 pr-12 py-3 text-sm resize-none focus:outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-400 shadow-sm placeholder-gray-400"
          />
          <div className="absolute right-2 top-1/2 -translate-y-1/2">
            {streaming ? (
              <button
                onClick={handleCancel}
                className="w-8 h-8 flex items-center justify-center rounded-lg bg-red-500 hover:bg-red-600 text-white transition-colors"
              >
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                  <rect x="6" y="6" width="12" height="12" rx="2" />
                </svg>
              </button>
            ) : (
              <button
                onClick={handleSend}
                disabled={!input.trim()}
                className="w-8 h-8 flex items-center justify-center rounded-lg bg-blue-500 hover:bg-blue-600 disabled:bg-gray-200 disabled:text-gray-400 text-white transition-colors"
              >
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M5 12h14M12 5l7 7-7 7" />
                </svg>
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
