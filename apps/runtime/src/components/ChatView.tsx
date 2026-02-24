import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { SkillManifest, ModelConfig, Message, StreamItem } from "../types";
import { ToolCallCard } from "./ToolCallCard";

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
        <code className={"bg-slate-600/50 px-1.5 py-0.5 rounded text-sm " + (className || "")} {...props}>
          {children}
        </code>
      );
    },
  };

  /** 渲染有序的 StreamItem 列表（文字和工具调用交替） */
  function renderStreamItems(items: StreamItem[], isStreaming: boolean) {
    return items.map((item, i) => {
      if (item.type === "tool_call" && item.toolCall) {
        return (
          <ToolCallCard
            key={`tc-${item.toolCall.id}`}
            toolCall={item.toolCall}
            subAgentBuffer={
              item.toolCall.name === "task" && item.toolCall.status === "running"
                ? subAgentBuffer
                : undefined
            }
          />
        );
      }
      if (item.type === "text" && item.content) {
        return (
          <div key={`txt-${i}`}>
            <ReactMarkdown components={markdownComponents}>{item.content}</ReactMarkdown>
          </div>
        );
      }
      return null;
    });
  }

  return (
    <div className="flex flex-col h-full">
      {/* 头部 */}
      <div className="flex items-center justify-between px-6 py-3 border-b border-slate-700 bg-slate-800">
        <div className="flex items-center">
          <span className="font-medium">{skill.name}</span>
          <span className="text-xs text-slate-400 ml-2">v{skill.version}</span>
          {workDir && (
            <span className="text-xs text-slate-500 ml-3 truncate max-w-[200px]" title={workDir}>
              {workDir.split(/[/\\]/).pop()}
            </span>
          )}
        </div>
        {currentModel && (
          <span className="text-xs text-slate-400">{currentModel.name}</span>
        )}
      </div>

      {/* 消息列表 */}
      <div className="flex-1 overflow-y-auto p-6 space-y-4">
        {agentState && (
          <div className="sticky top-0 z-10 flex items-center gap-2 bg-slate-800/90 backdrop-blur px-4 py-2 rounded-lg text-xs text-slate-300 border border-slate-700">
            <span className="animate-spin h-3 w-3 border-2 border-blue-400 border-t-transparent rounded-full" />
            {agentState.state === "thinking" && "思考中..."}
            {agentState.state === "tool_calling" && `执行工具: ${agentState.detail}`}
            {agentState.state === "error" && (
              <span className="text-red-400">错误: {agentState.detail}</span>
            )}
            <span className="text-slate-500 ml-auto">迭代 {agentState.iteration}</span>
          </div>
        )}
        {messages.map((m, i) => (
          <div key={i} className={"flex " + (m.role === "user" ? "justify-end" : "justify-start")}>
            <div
              className={
                "max-w-[80%] rounded-lg px-4 py-2 text-sm " +
                (m.role === "user"
                  ? "bg-blue-600 text-white"
                  : "bg-slate-700 text-slate-100")
              }
            >
              {m.role === "assistant" && m.streamItems ? (
                // 新格式：有序渲染
                renderStreamItems(m.streamItems, false)
              ) : m.role === "assistant" && m.toolCalls ? (
                // 旧格式兼容：工具在前，文字在后
                <>
                  <div className="mb-2">
                    {m.toolCalls.map((tc) => (
                      <ToolCallCard key={tc.id} toolCall={tc} />
                    ))}
                  </div>
                  <ReactMarkdown components={markdownComponents}>{m.content}</ReactMarkdown>
                </>
              ) : m.role === "assistant" ? (
                <ReactMarkdown components={markdownComponents}>{m.content}</ReactMarkdown>
              ) : (
                m.content
              )}
            </div>
          </div>
        ))}
        {/* 流式输出区域：按时间顺序渲染 */}
        {streamItems.length > 0 && (
          <div className="flex justify-start">
            <div className="max-w-[80%] bg-slate-700 rounded-lg px-4 py-2 text-sm text-slate-100">
              {renderStreamItems(streamItems, true)}
              <span className="animate-pulse">|</span>
            </div>
          </div>
        )}
        {/* AskUser 问答卡片 */}
        {askUserQuestion && (
          <div className="flex justify-start">
            <div className="max-w-[80%] bg-amber-900/40 border border-amber-600/50 rounded-lg px-4 py-3 text-sm">
              <div className="font-medium text-amber-200 mb-2">{askUserQuestion}</div>
              {askUserOptions.length > 0 && (
                <div className="flex flex-wrap gap-2 mb-2">
                  {askUserOptions.map((opt, i) => (
                    <button
                      key={i}
                      onClick={() => handleAnswerUser(opt)}
                      className="bg-amber-700/50 hover:bg-amber-600/50 text-amber-100 px-3 py-1 rounded text-xs transition-colors"
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
                  className="flex-1 bg-slate-700 border border-slate-600 rounded px-2 py-1 text-xs focus:outline-none focus:border-amber-500"
                />
                <button
                  onClick={() => handleAnswerUser(askUserAnswer)}
                  disabled={!askUserAnswer.trim()}
                  className="bg-amber-600 hover:bg-amber-700 disabled:bg-slate-600 px-3 py-1 rounded text-xs transition-colors"
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
            <div className="max-w-[80%] bg-orange-900/40 border border-orange-600/50 rounded-lg px-4 py-3 text-sm">
              <div className="font-medium text-orange-200 mb-2">需要确认</div>
              <div className="text-slate-300 mb-1">
                工具: <span className="text-orange-100 font-mono">{toolConfirm.toolName}</span>
              </div>
              <pre className="bg-slate-800/60 rounded p-2 text-xs text-slate-300 mb-3 overflow-x-auto max-h-40 overflow-y-auto">
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
          {streaming ? (
            <button
              onClick={handleCancel}
              className="bg-red-600 hover:bg-red-700 px-4 rounded text-sm font-medium transition-colors"
            >
              停止
            </button>
          ) : (
            <button
              onClick={handleSend}
              disabled={!input.trim()}
              className="bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 px-4 rounded text-sm font-medium transition-colors"
            >
              发送
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
