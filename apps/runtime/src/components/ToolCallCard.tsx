import { useState } from "react";
import { ToolCallInfo } from "../types";

const TOOL_ICONS: Record<string, string> = {
  read_file: "\u{1F4C2}",
  write_file: "\u{1F4DD}",
  glob: "\u{1F50D}",
  grep: "\u{1F50E}",
  bash: "\u{1F4BB}",
  sidecar_bridge: "\u{1F310}",
};

interface Props {
  toolCall: ToolCallInfo;
}

export function ToolCallCard({ toolCall }: Props) {
  const [expanded, setExpanded] = useState(false);
  const icon = TOOL_ICONS[toolCall.name] || "\u{1F527}";

  const statusLabel =
    toolCall.status === "running" ? (
      <span className="text-blue-400 text-xs animate-pulse">执行中...</span>
    ) : toolCall.status === "completed" ? (
      <span className="text-green-400 text-xs">完成</span>
    ) : (
      <span className="text-red-400 text-xs">错误</span>
    );

  const inputSummary = Object.entries(toolCall.input)
    .map(([k, v]) => `${k}: ${typeof v === "string" ? v : JSON.stringify(v)}`)
    .join(", ");
  const shortSummary = inputSummary.length > 60 ? inputSummary.slice(0, 60) + "..." : inputSummary;

  return (
    <div className="my-1 border border-slate-600 rounded-md overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-2 px-3 py-1.5 text-xs bg-slate-800 hover:bg-slate-750 transition-colors text-left"
      >
        <span>{icon}</span>
        <span className="font-medium text-slate-200">{toolCall.name}</span>
        <span className="text-slate-400 truncate flex-1">{shortSummary}</span>
        {statusLabel}
        <span className="text-slate-500">{expanded ? "\u25BC" : "\u25B6"}</span>
      </button>
      {expanded && (
        <div className="px-3 py-2 bg-slate-900 text-xs space-y-2">
          <div>
            <div className="text-slate-400 mb-1">参数:</div>
            <pre className="bg-slate-950 rounded p-2 overflow-x-auto text-slate-300">
              {JSON.stringify(toolCall.input, null, 2)}
            </pre>
          </div>
          {toolCall.output && (
            <div>
              <div className="text-slate-400 mb-1">结果:</div>
              <pre className="bg-slate-950 rounded p-2 overflow-x-auto text-slate-300 max-h-40 overflow-y-auto">
                {toolCall.output}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
