import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  AgentEmployee,
  AgentProfileAnswerInput,
  AgentProfileDraft,
  AgentProfilePayload,
  ApplyAgentProfileResult,
} from "../../types";

interface Props {
  employee: AgentEmployee | null;
}

interface QuestionItem {
  key: string;
  label: string;
  prompt: string;
  placeholder: string;
}

const QUESTIONS: QuestionItem[] = [
  {
    key: "mission",
    label: "核心使命",
    prompt: "这个员工最核心的业务使命是什么？",
    placeholder: "例如：把需求推进到可上线交付，并对里程碑负责",
  },
  {
    key: "responsibilities",
    label: "关键职责",
    prompt: "它日常需要承担哪些关键职责？",
    placeholder: "例如：需求澄清、任务拆解、风险同步、验收把关",
  },
  {
    key: "collaboration",
    label: "协作方式",
    prompt: "它和其他员工/用户如何协作？",
    placeholder: "例如：先澄清上下文，再拆任务，阻塞时升级到主员工",
  },
  {
    key: "tone",
    label: "沟通风格",
    prompt: "你希望它的沟通语气是什么样？",
    placeholder: "例如：专业、简洁、结论先行",
  },
  {
    key: "boundaries",
    label: "边界规则",
    prompt: "有哪些明确不能越过的边界？",
    placeholder: "例如：不编造事实，高风险操作必须确认",
  },
  {
    key: "user_profile",
    label: "用户画像",
    prompt: "它服务的主要用户是谁？",
    placeholder: "例如：产品经理、销售、实施顾问",
  },
];

function buildAnswersMap(answers: Record<string, string>): AgentProfileAnswerInput[] {
  return QUESTIONS.map((question) => ({
    key: question.key,
    question: question.prompt,
    answer: (answers[question.key] || "").trim(),
  }));
}

export function AgentProfileChatWizard({ employee }: Props) {
  const [currentIndex, setCurrentIndex] = useState(0);
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [draft, setDraft] = useState<AgentProfileDraft | null>(null);
  const [applyResult, setApplyResult] = useState<ApplyAgentProfileResult | null>(null);
  const [running, setRunning] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    setCurrentIndex(0);
    setAnswers({});
    setDraft(null);
    setApplyResult(null);
    setMessage("");
  }, [employee?.id]);

  const currentQuestion = QUESTIONS[currentIndex];
  const answeredCount = useMemo(
    () => QUESTIONS.filter((q) => (answers[q.key] || "").trim().length > 0).length,
    [answers],
  );

  async function generatePreview() {
    if (!employee?.id) return;
    setRunning(true);
    setMessage("");
    setApplyResult(null);
    try {
      const payload: AgentProfilePayload = {
        employee_db_id: employee.id,
        answers: buildAnswersMap(answers),
      };
      const out = await invoke<AgentProfileDraft>("generate_agent_profile_draft", { payload });
      setDraft(out);
      setMessage("已生成预览，可检查后应用。");
    } catch (e) {
      setMessage(String(e));
    } finally {
      setRunning(false);
    }
  }

  async function applyProfile() {
    if (!employee?.id) return;
    setRunning(true);
    setMessage("");
    try {
      const payload: AgentProfilePayload = {
        employee_db_id: employee.id,
        answers: buildAnswersMap(answers),
      };
      const out = await invoke<ApplyAgentProfileResult>("apply_agent_profile", { payload });
      setApplyResult(out);
      const okCount = out.files.filter((item) => item.ok).length;
      setMessage(`${okCount}/${out.files.length} 文件写入成功`);
    } catch (e) {
      setMessage(String(e));
    } finally {
      setRunning(false);
    }
  }

  if (!employee) {
    return (
      <div className="rounded-lg border border-dashed border-gray-300 p-3 text-xs text-gray-500">
        先在左侧选择一个员工，再开始对话配置 AGENTS/SOUL/USER。
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-gray-200 p-3 space-y-3">
      <div className="flex items-center justify-between">
        <div className="text-xs font-medium text-gray-700">对话配置智能体（AGENTS/SOUL/USER）</div>
        <div className="text-[11px] text-gray-500">
          {answeredCount}/{QUESTIONS.length} 已回答
        </div>
      </div>

      <div className="rounded-md border border-gray-100 bg-gray-50 p-2 space-y-2">
        <div className="text-xs text-gray-600">
          问题 {currentIndex + 1}/{QUESTIONS.length} · {currentQuestion.label}
        </div>
        <div className="text-sm text-gray-900">{currentQuestion.prompt}</div>
        <textarea
          className="w-full border border-gray-200 rounded px-2 py-1.5 text-sm bg-white"
          rows={2}
          placeholder={currentQuestion.placeholder}
          value={answers[currentQuestion.key] || ""}
          onChange={(e) =>
            setAnswers((prev) => ({
              ...prev,
              [currentQuestion.key]: e.target.value,
            }))
          }
        />
        <div className="flex items-center gap-2">
          <button
            type="button"
            className="h-7 px-2 rounded border border-gray-200 text-xs disabled:opacity-40"
            onClick={() => setCurrentIndex((idx) => Math.max(0, idx - 1))}
            disabled={currentIndex === 0}
          >
            上一题
          </button>
          <button
            type="button"
            className="h-7 px-2 rounded border border-gray-200 text-xs disabled:opacity-40"
            onClick={() => setCurrentIndex((idx) => Math.min(QUESTIONS.length - 1, idx + 1))}
            disabled={currentIndex === QUESTIONS.length - 1}
          >
            下一题
          </button>
          <button
            type="button"
            className="h-7 px-2 rounded bg-blue-500 text-white text-xs disabled:bg-blue-300"
            onClick={generatePreview}
            disabled={running}
          >
            生成预览
          </button>
          <button
            type="button"
            className="h-7 px-2 rounded bg-emerald-500 text-white text-xs disabled:bg-emerald-300"
            onClick={applyProfile}
            disabled={running || !draft}
          >
            应用到员工目录
          </button>
        </div>
      </div>

      {message && (
        <div className="text-xs text-blue-700 bg-blue-50 border border-blue-100 rounded px-2 py-1">
          {message}
        </div>
      )}

      {draft && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-2">
          <div className="border border-gray-100 rounded p-2">
            <div className="text-xs font-medium text-gray-700 mb-1">AGENTS.md</div>
            <pre className="text-[11px] text-gray-600 whitespace-pre-wrap max-h-40 overflow-y-auto">
              {draft.agents_md}
            </pre>
          </div>
          <div className="border border-gray-100 rounded p-2">
            <div className="text-xs font-medium text-gray-700 mb-1">SOUL.md</div>
            <pre className="text-[11px] text-gray-600 whitespace-pre-wrap max-h-40 overflow-y-auto">
              {draft.soul_md}
            </pre>
          </div>
          <div className="border border-gray-100 rounded p-2">
            <div className="text-xs font-medium text-gray-700 mb-1">USER.md</div>
            <pre className="text-[11px] text-gray-600 whitespace-pre-wrap max-h-40 overflow-y-auto">
              {draft.user_md}
            </pre>
          </div>
        </div>
      )}

      {applyResult && (
        <div className="space-y-1">
          {applyResult.files.map((item) => (
            <div key={item.path} className="text-[11px] text-gray-600">
              {item.ok ? "OK" : "FAIL"} · {item.path}
              {!item.ok && item.error ? ` · ${item.error}` : ""}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
