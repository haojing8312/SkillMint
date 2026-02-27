import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { SkillManifest, ModelConfig, Message, StreamItem, FileAttachment, SkillRouteEvent } from "../types";
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
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const subAgentBufferRef = useRef("");

  // File Upload: 附件状态
  const [attachedFiles, setAttachedFiles] = useState<FileAttachment[]>([]);
  const MAX_FILE_SIZE = 5 * 1024 * 1024; // 5MB
  const MAX_FILES = 5;

  // 右侧面板状态
  const [sidePanelOpen, setSidePanelOpen] = useState(false);
  const [sidePanelTab, setSidePanelTab] = useState<"assets" | "routing">("assets");
  const [routeEvents, setRouteEvents] = useState<SkillRouteEvent[]>([]);

  // File Upload: 读取文件为文本
  const readFileAsText = (file: File): Promise<string> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result as string);
      reader.onerror = reject;
      reader.readAsText(file);
    });
  };

  // File Upload: 处理文件选择
  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);

    if (attachedFiles.length + files.length > MAX_FILES) {
      alert(`最多只能上传 ${MAX_FILES} 个文件`);
      return;
    }

    const newFiles: FileAttachment[] = [];
    for (const file of files) {
      if (file.size > MAX_FILE_SIZE) {
        alert(`文件 ${file.name} 超过 5MB 限制`);
        continue;
      }

      const content = await readFileAsText(file);
      newFiles.push({
        name: file.name,
        size: file.size,
        type: file.type,
        content,
      });
    }

    setAttachedFiles((prev) => [...prev, ...newFiles]);
    e.target.value = ""; // 重置 input
  };

  // File Upload: 删除附件
  const removeAttachedFile = (index: number) => {
    setAttachedFiles((prev) => prev.filter((_, i) => i !== index));
  };

  // Secure Workspace: 工作空间状态
  const [workspace, setWorkspace] = useState<string>("");

  // Secure Workspace: 加载会话的工作空间
  const loadWorkspace = async (sid: string) => {
    try {
      const sessions = await invoke<any[]>("get_sessions", { skillId: skill.id });
      const current = sessions.find((s: any) => s.id === sid);
      if (current) {
        setWorkspace(current.work_dir || "");
      }
    } catch (e) {
      console.error("加载工作空间失败:", e);
    }
  };

  // Secure Workspace: 更新会话的工作空间
  const updateWorkspace = async (newWorkspace: string) => {
    try {
      await invoke("update_session_workspace", {
        sessionId,
        workspace: newWorkspace,
      });
      setWorkspace(newWorkspace);
    } catch (e) {
      console.error("更新工作空间失败:", e);
    }
  };

  // Manual Compression: 压缩状态
  const [compacting, setCompacting] = useState(false);

  // Manual Compression: 处理压缩
  const handleCompact = async () => {
    if (compacting || !sessionId) return;
    setCompacting(true);
    try {
      const result = await invoke<{
        original_tokens: number;
        new_tokens: number;
        summary: string;
      }>("compact_context", { sessionId });

      // 显示压缩结果
      const summaryText = `📦 上下文已压缩：${result.original_tokens} → ${result.new_tokens} tokens`;

      // 添加系统消息
      setMessages((prev) => [
        ...prev,
        { role: "system", content: summaryText, created_at: new Date().toISOString() },
        { role: "assistant", content: result.summary, created_at: new Date().toISOString() },
      ]);

      // 刷新消息
      await loadMessages(sessionId);
    } catch (e) {
      console.error("压缩失败:", e);
      alert("压缩失败: " + String(e));
    } finally {
      setCompacting(false);
    }
  };

  // sessionId 变化时加载历史消息
  useEffect(() => {
    loadMessages(sessionId);
    loadWorkspace(sessionId);
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
    setRouteEvents([]);
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

  // skill-route-node-updated 事件监听：自动路由调用链
  useEffect(() => {
    const unlistenPromise = listen<SkillRouteEvent>("skill-route-node-updated", ({ payload }) => {
      if (payload.session_id !== sessionId) return;
      setRouteEvents((prev) => {
        const idx = prev.findIndex((e) => e.node_id === payload.node_id);
        if (idx >= 0) {
          const next = [...prev];
          next[idx] = payload;
          return next;
        }
        return [...prev, payload];
      });
    });
    return () => {
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
    // 检查是否是 /compact 命令
    if (input.trim() === "/compact") {
      setInput("");
      handleCompact();
      return;
    }

    if (!input.trim() && attachedFiles.length === 0) return;
    if (streaming || !sessionId) return;

    // 构建消息内容：用户输入 + 附件
    const msg = input.trim();
    let fullContent = msg;

    if (attachedFiles.length > 0) {
      const attachmentsText = attachedFiles.map((f) => {
        const ext = f.name.split(".").pop()?.toLowerCase() || "";
        const isImage = f.type.startsWith("image/");
        if (isImage) {
          return `## ${f.name}\n![${f.name}](${f.content})`;
        }
        return `## ${f.name}\n\`\`\`${ext}\n${f.content}\n\`\`\``;
      }).join("\n\n");

      fullContent = msg
        ? `${msg}\n\n---\n\n附件文件：\n${attachmentsText}`
        : `附件文件：\n${attachmentsText}`;
    }

    setInput("");
    setAttachedFiles([]); // 发送后清空附件
    setMessages((prev) => [
      ...prev,
      { role: "user", content: fullContent, created_at: new Date().toISOString() },
    ]);
    setStreaming(true);
    streamItemsRef.current = [];
    setStreamItems([]);
    subAgentBufferRef.current = "";
    setSubAgentBuffer("");
    try {
      await invoke("send_message", { sessionId, userMessage: fullContent });
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
  const routeCompleted = routeEvents.filter((e) => e.status === "completed").length;
  const routeFailed = routeEvents.filter((e) => e.status === "failed").length;
  const routeTotalDuration = routeEvents.reduce((sum, e) => sum + (e.duration_ms || 0), 0);

  // Markdown 渲染组件配置
  const markdownComponents = {
    // 代码块
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
        <code className={"bg-gray-200/60 px-1.5 py-0.5 rounded text-sm text-gray-800 font-mono " + (className || "")} {...props}>
          {children}
        </code>
      );
    },
    // 标题
    h1: ({ children }: any) => <h1 className="text-2xl font-bold text-gray-900 mt-6 mb-3 pb-2 border-b border-gray-200">{children}</h1>,
    h2: ({ children }: any) => <h2 className="text-xl font-bold text-gray-900 mt-5 mb-2.5 pb-1.5 border-b border-gray-100">{children}</h2>,
    h3: ({ children }: any) => <h3 className="text-lg font-semibold text-gray-800 mt-4 mb-2">{children}</h3>,
    h4: ({ children }: any) => <h4 className="text-base font-semibold text-gray-700 mt-3 mb-1.5">{children}</h4>,
    h5: ({ children }: any) => <h5 className="text-sm font-semibold text-gray-700 mt-2 mb-1">{children}</h5>,
    h6: ({ children }: any) => <h6 className="text-sm font-medium text-gray-600 mt-2 mb-1">{children}</h6>,
    // 段落
    p: ({ children }: any) => <p className="text-sm text-gray-700 leading-relaxed mb-3">{children}</p>,
    // 列表
    ul: ({ children }: any) => <ul className="list-disc list-inside space-y-1 mb-3 text-sm text-gray-700">{children}</ul>,
    ol: ({ children }: any) => <ol className="list-decimal list-inside space-y-1 mb-3 text-sm text-gray-700">{children}</ol>,
    li: ({ children }: any) => <li className="text-sm text-gray-700">{children}</li>,
    // 链接
    a: ({ href, children }: any) => (
      <a
        href={href}
        className="text-blue-500 hover:text-blue-600 underline underline-offset-2 text-sm"
        target="_blank"
        rel="noopener noreferrer"
      >
        {children}
      </a>
    ),
    // 引用块
    blockquote: ({ children }: any) => (
      <blockquote className="border-l-4 border-gray-300 pl-4 py-1 my-3 bg-gray-50 rounded-r-lg">
        <div className="text-sm text-gray-600 italic">{children}</div>
      </blockquote>
    ),
    // 表格
    table: ({ children }: any) => (
      <div className="overflow-x-auto my-3">
        <table className="min-w-full border border-gray-200 rounded-lg overflow-hidden text-sm">{children}</table>
      </div>
    ),
    thead: ({ children }: any) => <thead className="bg-gray-100">{children}</thead>,
    tbody: ({ children }: any) => <tbody className="divide-y divide-gray-100">{children}</tbody>,
    tr: ({ children }: any) => <tr className="hover:bg-gray-50">{children}</tr>,
    th: ({ children }: any) => (
      <th className="px-3 py-2 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider bg-gray-50">
        {children}
      </th>
    ),
    td: ({ children }: any) => <td className="px-3 py-2 text-sm text-gray-700">{children}</td>,
    // 水平线
    hr: () => <hr className="my-6 border-gray-200" />,
    // 强调
    strong: ({ children }: any) => <strong className="font-semibold text-gray-900">{children}</strong>,
    em: ({ children }: any) => <em className="italic text-gray-700">{children}</em>,
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
      <div className="flex items-center justify-between px-6 py-3 border-b border-gray-200 bg-white/70 backdrop-blur-sm">
        <div className="flex items-center gap-3 min-w-0">
          <span className="font-semibold text-gray-900 flex-shrink-0">{skill.name}</span>
        </div>
        <div className="flex items-center gap-3 flex-shrink-0">
          {/* 右侧面板切换按钮 */}
          <button
            onClick={() => setSidePanelOpen(!sidePanelOpen)}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs transition-colors ${
              sidePanelOpen
                ? "bg-blue-100 text-blue-600"
                : "bg-gray-100 hover:bg-gray-200 text-gray-600"
            }`}
          >
            <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2" />
            </svg>
            面板
          </button>
          {/* Secure Workspace 选择器 */}
          <button
            onClick={() => {
              // 打开目录选择器
              invoke<string | null>("select_directory", {
                defaultPath: workspace || undefined,
              }).then((newDir) => {
                if (newDir) {
                  updateWorkspace(newDir);
                }
              });
            }}
            className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-gray-100 hover:bg-gray-200 text-xs text-gray-600 transition-colors"
          >
            <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <span className="max-w-[150px] truncate">
              {workspace || "选择工作目录"}
            </span>
          </button>
          {currentModel && (
            <span className="text-xs text-gray-500 bg-gray-100 px-2 py-0.5 rounded-md">
              {currentModel.name}
            </span>
          )}
        </div>
      </div>

      {/* 主内容区：消息列表 + 右侧面板 */}
      <div className="flex-1 flex overflow-hidden">
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
              {/* 光标闪烁效果 */}
              <span className="inline-block w-0.5 h-4 bg-blue-400 ml-0.5 align-middle animate-[blink_1s_infinite]" />
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

      {/* 右侧面板 */}
      <AnimatePresence>
        {sidePanelOpen && (
          <motion.div
            initial={{ width: 0, opacity: 0 }}
            animate={{ width: 320, opacity: 1 }}
            exit={{ width: 0, opacity: 0 }}
            transition={{ type: "spring", stiffness: 300, damping: 30 }}
            className="h-full bg-gray-50 border-l border-gray-200 overflow-hidden flex flex-col"
          >
            <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 bg-white/50">
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setSidePanelTab("assets")}
                  className={`px-2 py-1 rounded text-xs transition-colors ${
                    sidePanelTab === "assets" ? "bg-blue-100 text-blue-600" : "text-gray-500 hover:bg-gray-100"
                  }`}
                >
                  附件与工具
                </button>
                <button
                  onClick={() => setSidePanelTab("routing")}
                  className={`px-2 py-1 rounded text-xs transition-colors ${
                    sidePanelTab === "routing" ? "bg-blue-100 text-blue-600" : "text-gray-500 hover:bg-gray-100"
                  }`}
                >
                  自动路由
                </button>
              </div>
              <button
                onClick={() => setSidePanelOpen(false)}
                className="p-1 hover:bg-gray-100 rounded"
              >
                <svg className="w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              {sidePanelTab === "routing" && (
                <div className="space-y-3">
                  <div className="p-3 bg-white rounded-lg border border-gray-200 shadow-sm">
                    <div className="text-xs text-gray-500 mb-2">概览</div>
                    <div className="grid grid-cols-2 gap-2 text-xs">
                      <div className="p-2 rounded bg-gray-50 text-gray-600">总节点: {routeEvents.length}</div>
                      <div className="p-2 rounded bg-gray-50 text-green-600">成功: {routeCompleted}</div>
                      <div className="p-2 rounded bg-gray-50 text-red-500">失败: {routeFailed}</div>
                      <div className="p-2 rounded bg-gray-50 text-gray-600">总耗时: {routeTotalDuration}ms</div>
                    </div>
                  </div>

                  <div>
                    <div className="text-xs font-medium text-gray-500 mb-2">调用链</div>
                    {routeEvents.length === 0 ? (
                      <div className="text-center text-gray-400 text-sm py-6">暂无路由事件</div>
                    ) : (
                      <div className="space-y-2">
                        {routeEvents.map((evt) => (
                          <div
                            key={evt.node_id}
                            className="p-3 bg-white rounded-lg border border-gray-200 shadow-sm"
                          >
                            <div className="flex items-center justify-between mb-1">
                              <span className="text-sm font-medium text-gray-700 font-mono">{evt.skill_name || "(unknown)"}</span>
                              <span
                                className={`text-[11px] px-1.5 py-0.5 rounded ${
                                  evt.status === "completed"
                                    ? "bg-green-100 text-green-700"
                                    : evt.status === "failed"
                                    ? "bg-red-100 text-red-600"
                                    : "bg-blue-100 text-blue-600"
                                }`}
                              >
                                {evt.status}
                              </span>
                            </div>
                            <div className="text-[11px] text-gray-500 space-y-0.5">
                              <div>depth: {evt.depth}</div>
                              <div>node: {evt.node_id}</div>
                              {evt.parent_node_id && <div>parent: {evt.parent_node_id}</div>}
                              {typeof evt.duration_ms === "number" && <div>duration: {evt.duration_ms}ms</div>}
                              {evt.error_code && <div className="text-red-500">error: {evt.error_code}</div>}
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              )}

              {sidePanelTab === "assets" && (
                <>
              {/* 附件列表 */}
              {attachedFiles.length > 0 && (
                <div>
                  <div className="text-xs font-medium text-gray-500 mb-2">附件 ({attachedFiles.length})</div>
                  <div className="space-y-2">
                    {attachedFiles.map((file, index) => (
                      <div
                        key={index}
                        className="p-3 bg-white rounded-lg border border-gray-200 shadow-sm"
                      >
                        <div className="flex items-center justify-between mb-2">
                          <div className="flex items-center gap-2">
                            <svg className="w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                              <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                            </svg>
                            <span className="text-sm font-medium text-gray-700 truncate max-w-[180px]">{file.name}</span>
                          </div>
                          <button
                            onClick={() => removeAttachedFile(index)}
                            className="p-1 hover:bg-gray-100 rounded"
                          >
                            <svg className="w-3.5 h-3.5 text-gray-400 hover:text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                          </button>
                        </div>
                        <div className="text-xs text-gray-400">{(file.size / 1024).toFixed(1)} KB</div>
                        {/* 文件内容预览（只显示前200字符） */}
                        {file.content.length > 0 && (
                          <div className="mt-2 p-2 bg-gray-50 rounded text-xs text-gray-600 font-mono max-h-24 overflow-y-auto">
                            {file.content.slice(0, 200)}
                            {file.content.length > 200 && "..."}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* 工具调用历史（从消息中提取） */}
              {messages.some(m => m.toolCalls && m.toolCalls.length > 0) && (
                <div>
                  <div className="text-xs font-medium text-gray-500 mb-2">工具调用</div>
                  <div className="space-y-2">
                    {messages.flatMap((m, mi) =>
                      (m.toolCalls || []).map((tc, ti) => (
                        <div
                          key={`${mi}-${ti}`}
                          className="p-3 bg-white rounded-lg border border-gray-200 shadow-sm"
                        >
                          <div className="flex items-center gap-2 mb-1">
                            {tc.status === "completed" ? (
                              <svg className="w-3.5 h-3.5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={3}>
                                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                              </svg>
                            ) : tc.status === "error" ? (
                              <svg className="w-3.5 h-3.5 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                                <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                              </svg>
                            ) : (
                              <span className="h-2.5 w-2.5 rounded-full bg-blue-400 animate-pulse" />
                            )}
                            <span className="text-sm font-medium text-gray-700 font-mono">{tc.name}</span>
                          </div>
                          {tc.output && (
                            <div className="mt-2 p-2 bg-gray-50 rounded text-xs text-gray-600 font-mono max-h-24 overflow-y-auto">
                              {tc.output.slice(0, 200)}
                              {tc.output.length > 200 && "..."}
                            </div>
                          )}
                        </div>
                      ))
                    )}
                  </div>
                </div>
              )}

              {/* 空状态 */}
              {attachedFiles.length === 0 && !messages.some(m => m.toolCalls && m.toolCalls.length > 0) && (
                <div className="text-center text-gray-400 text-sm py-8">
                  暂无附件和工具调用
                </div>
              )}
                </>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
      </div>

      {/* 输入区域 */}
      <div className="px-6 py-3 bg-gray-50/80">
        {routeEvents.length > 0 && (
          <div className="max-w-3xl mx-auto mb-2">
            <button
              onClick={() => {
                setSidePanelOpen(true);
                setSidePanelTab("routing");
              }}
              className="w-full text-left px-3 py-2 rounded-xl bg-white border border-gray-200 text-xs text-gray-600 hover:bg-gray-50 transition-colors"
            >
              已自动路由 {routeEvents.length} 个子 Skill · 成功 {routeCompleted} · 失败 {routeFailed} · {routeTotalDuration}ms
            </button>
          </div>
        )}
        <div className="max-w-3xl mx-auto bg-white border border-gray-200 rounded-2xl shadow-sm focus-within:border-blue-400 focus-within:ring-1 focus-within:ring-blue-400 transition-all">
          {/* 隐藏的文件输入 */}
          <input
            type="file"
            multiple
            onChange={handleFileSelect}
            className="hidden"
            id="file-upload"
          />

          {/* 输入框主体 */}
          <textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              // auto-expand
              const el = e.target;
              el.style.height = "auto";
              el.style.height = Math.min(el.scrollHeight, 200) + "px";
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder="输入消息，Shift+Enter 换行..."
            rows={3}
            className="w-full bg-transparent pl-4 pr-4 pt-3 pb-2 text-sm resize-none focus:outline-none placeholder-gray-400 min-h-[80px] max-h-[200px]"
          />
          {/* 底部工具栏 */}
          <div className="flex items-center justify-between px-3 pb-2.5">
            <div className="flex items-center gap-2 text-xs text-gray-400">
              {skill.description && (
                <span className="truncate max-w-[300px]" title={skill.description}>
                  {skill.description}
                </span>
              )}
            </div>
            <div className="flex items-center gap-2">
              {/* 附件按钮 */}
              <label
                htmlFor="file-upload"
                className="h-8 px-3 flex items-center justify-center gap-1.5 rounded-lg bg-gray-100 hover:bg-gray-200 active:scale-[0.97] text-gray-600 text-xs font-medium transition-all cursor-pointer"
              >
                <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
                </svg>
                附件
              </label>
              {/* 压缩按钮 */}
              <button
                onClick={handleCompact}
                disabled={compacting}
                className="h-8 px-3 flex items-center justify-center gap-1.5 rounded-lg bg-gray-100 hover:bg-gray-200 active:scale-[0.97] disabled:opacity-50 text-gray-600 text-xs font-medium transition-all"
              >
                <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                </svg>
                {compacting ? "压缩中..." : "压缩"}
              </button>
              {streaming ? (
                <button
                  onClick={handleCancel}
                  className="h-8 px-3 flex items-center justify-center gap-1.5 rounded-lg bg-red-500 hover:bg-red-600 active:scale-[0.97] text-white text-xs font-medium transition-all"
                >
                  <svg className="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
                    <rect x="6" y="6" width="12" height="12" rx="2" />
                  </svg>
                  停止
                </button>
              ) : (
                <button
                  onClick={handleSend}
                  disabled={!input.trim() && attachedFiles.length === 0}
                  className="h-8 px-3 flex items-center justify-center gap-1.5 rounded-lg bg-blue-500 hover:bg-blue-600 active:scale-[0.97] disabled:bg-gray-100 disabled:text-gray-400 text-white text-xs font-medium transition-all"
                >
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M5 12h14M12 5l7 7-7 7" />
                  </svg>
                  发送
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
